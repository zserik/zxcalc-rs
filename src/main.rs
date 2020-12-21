mod scv;
#[macro_use]
mod ssv;

mod cpm_;
mod parser;
mod plugins;

fn eval_op(op: char, mut y: f64, x: f64) -> f64 {
    match op {
        '+' => y += x,
        '-' => y -= x,
        '*' => y *= x,
        '/' => y /= x,
        '%' => y = plugins::zx_modulo(y, x),
        _ => {}
    }
    y
}

fn getline_wprompt(prompt: &str) -> Option<String> {
    if atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout) {
        use std::io::Write;
        let mut stdout = std::io::stdout();
        write!(stdout, "{}", prompt).ok()?;
        stdout.flush().ok()?;
    }
    let txt: String = text_io::read!("{}\n");
    Some(txt.trim().to_string())
}

use parser::*;

fn main() {
    let mut value: f64 = 0.0;
    let mut cpm = cpm_::CalcPluginManager::new();
    let mut vars = std::collections::HashMap::<String, f64>::new();

    while let Some(curin) = getline_wprompt("zxcalc > ") {
        let mut got_error = false;
        let mut breakout = false;

        for i in parse_line(&curin, value) {
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

            eprintln!("run {:?}", i);
            match i {
                XNode::Error(err) => {
                    eprintln!("\tERROR: {:?}", err);
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

                XNode::SetScale(clp, _) => {
                    // we don't want a "\t0\n" line after this
                    got_error = true;
                    let mut scval = resolved_val.take().unwrap();
                    if scval > 0.0 && scval < 1.0 {
                        scval = -(1.0 / scval);
                    }
                    cpm.set_scale(&clp, scval.round() as i64);
                }

                XNode::Assign(varname) => {
                    vars.insert(varname, value);
                }

                XNode::Calc(op, clp, _) => {
                    let xval = resolved_val.take().unwrap();
                    if let Some(res) = cpm.calc(&clp, xval) {
                        value = eval_op(op, value, res);
                    } else {
                        got_error = true;
                        eprintln!("\tERROR: {}: calc failed", &clp);
                    }
                }

                XNode::CalcInv(clp) => {
                    if let Some(res) = cpm.calcinv(&clp, value) {
                        println!("calcinv {} {} -> {}", clp, value, res);
                        value = res;
                    } else {
                        got_error = true;
                        eprintln!("\tERROR: {}: calcinv failed", &clp);
                    }
                }
            }
            eprintln!("\t-> {}", value);
        }

        if !got_error {
            println!("\t{}", value);
        }
        if breakout {
            break;
        }
    }
}
