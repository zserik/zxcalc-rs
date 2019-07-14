#[macro_use]
extern crate failure;
extern crate hashbrown;
#[macro_use]
extern crate lazy_static;
extern crate libloading;
extern crate sortedvec;
extern crate termion;
extern crate zxcalc_common;

mod scv;
#[macro_use]
mod ssv;

mod cpm_;
mod gl;
mod parser;

fn eval_op(op: char, mut y: f64, x: f64) -> f64 {
    match op {
        '+' => y += x,
        '-' => y -= x,
        '*' => y *= x,
        '/' => y /= x,
        '%' => y = zxcalc_common::zx_modulo(y, x),
        _ => {}
    }
    y
}

use parser::*;

fn main() {
    let mut value: f64 = 0.0;
    let mut cpm = cpm_::CalcPluginManager::new();
    let mut vars = hashbrown::HashMap::<String, f64>::new();

    while let Some(curin) = gl::getline_wprompt("zxcalc > ") {
        let mut got_error = false;
        let mut breakout = false;

        for i in parse_line(&curin, value) {
            let mut resolved_clp: Option<std::borrow::Cow<str>> = {
                match &i {
                    XNode::Calc(_, clp, _) | XNode::CalcInv(clp) | XNode::SetScale(clp, _) => {
                        if let Some(clp2) = cpm.resolve(&clp) {
                            Some(clp2)
                        } else {
                            eprintln!("\tERROR: {}: unable to resolve plugin name", &clp);
                            got_error = true;
                            break;
                        }
                    }
                    _ => None,
                }
            };

            let mut resolved_val: Option<f64> = {
                match &i {
                    XNode::Calc(_, _, xval) | XNode::SetScale(_, xval) => match xval {
                        XValue::Imm(x) => Some(*x),
                        XValue::Var(x) => {
                            if let Some(y) = vars.get(x) {
                                Some(*y)
                            } else {
                                eprintln!("\tERROR: {}: unknown variable", &x);
                                got_error = true;
                                break;
                            }
                        }
                    },
                    _ => None,
                }
            };

            match i {
                XNode::Error(err) => {
                    eprintln!("\tERROR:");
                    for cause in err.iter_chain() {
                        eprintln!("< {}", cause);
                    }
                    got_error = true;
                }
                XNode::Cmd(cmd) => match &cmd[..] {
                    "exit" | "quit" => breakout = true,
                    "list-loaded-plugins" => cpm.list_loaded_plugins(),
                    "help" => {
                        println!("  --COMMANDS--
\tquit\t\t\t\texit this program
\thelp\t\t\t\tprint this text
\tlist-loaded-plugins\t\tprint list of currently loaded plugins
\tset-scale PLG ['+'|'-']NUM\tset the scaling factor for the plugin (NUM < 0 --> division)

  --SYNTAX--
\tThis program expects input lines not containing one of the
\tcommands above to have the following format:
\t\t(('+'|'-'|'*'|'/') PLG ['+'|'-']NUM|':' PLG|'=' VAR)*

");
                    }
                    _ => eprintln!("\tERROR: unknown command {}\n", cmd),
                },

                XNode::SetScale(_, _) => {
                    // we don't want a "\t0\n" line after this
                    got_error = true;
                    let mut scval = resolved_val.take().unwrap();
                    if scval > 0.0 && scval < 1.0 {
                        scval = -(1.0 / scval);
                    }
                    cpm.set_scale(&resolved_clp.take().unwrap(), scval);
                }

                XNode::Assign(varname) => {
                    vars.insert(varname, value);
                }

                XNode::Calc(op, _, _) => {
                    let clp = resolved_clp.take().unwrap();
                    let xval = resolved_val.take().unwrap();
                    if let Some(res) = cpm.calc(&clp, xval) {
                        value = eval_op(op, value, res);
                    } else {
                        got_error = true;
                        eprintln!("\tERROR: {} {}: calc failed", &clp, xval);
                    }
                }

                _ => {
                    got_error = true;
                    eprintln!("\tERROR: unimplemented operation");
                }
            }
        }

        if !got_error {
            println!("\t{}", value);
        }
        if breakout {
            break;
        }
    }
}
