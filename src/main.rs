#[macro_use]
extern crate failure;
extern crate hashbrown;
#[macro_use]
extern crate indoc;
#[macro_use]
extern crate lazy_static;
extern crate libloading;
extern crate sortedvec;
extern crate termion;
extern crate zxcalc_common;

mod cpm;
mod scv;
#[macro_use]
mod ssv;

type VarName = String;

#[derive(Clone, Debug, PartialEq)]
enum XValue {
    Var(VarName),
    Imm(f64),
}

type PluginName = String;

#[derive(Debug)]
enum XNode {
    Assign(VarName),
    Calc(char, PluginName, XValue),
    CalcInv(PluginName),
    Cmd(String),
    SetScale(PluginName, XValue),
    Error(failure::Error),
}

#[derive(Debug, Fail)]
enum XParseError {
    #[fail(display = "invalid invocation of '{}': USAGE: {} {}", cmd, cmd, usage)]
    InvalidInvocationError {
        cmd: &'static str,
        usage: &'static str,
    },

    #[fail(display = "expected one of {} instead of '{}'", expected, got)]
    UnexpectedTokenError {
        expected: &'static str,
        got: String,
    },

    #[fail(display = "unexpected EOL while looking for {}", expected)]
    UnexpectedEOLWhileError {
        expected: &'static str,
    },

    #[fail(display = "token no. {} is unexpected empty", pos)]
    UnexpectedEmptyError {
        pos: usize,
    },
}

macro_rules! xnode_from_perror {
    ($et:ident { $($enam:ident : $val:expr),+ }) => {
        XNode::Error((XParseError::$et { $($enam: $val,)+ }).into())
    }
}

use scv::{SCV, scv_contains, scv_create};
use ssv::{SSV, ssv_contains};

lazy_static! {
    static ref CMDS2PASSUP: SSV = ssv_create!(
        "exit",
        "help",
        "list-loaded-plugins",
        "quit"
    );
    static ref ALL_OPERATORS: SCV = scv_create("+-*/:=");
}

impl std::convert::From<&str> for XValue {
    fn from(s: &str) -> Self {
        match s.parse::<f64>() {
            Ok(x) => XValue::Imm(x),
            Err(_) => XValue::Var(s.to_owned()),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum ParserMode {
    Start,
    Clp,
    // ClpIX ? ClpInv : Assign
    ClpIX(bool),
    Num,
}

fn parse_line(line: &str, prev_val: f64) -> Vec<XNode> {
    let mut toks: Vec<_> = line.split_whitespace().collect();
    if toks.is_empty() {
        return Vec::new();
    }
    let mut nodes = Vec::<XNode>::new();

    {
        let &i = toks.first().unwrap();
        if i.is_empty() {
            return vec![xnode_from_perror!(UnexpectedEmptyError { pos: 0 })];
        }

        if toks.len() == 1 && ssv_contains(&CMDS2PASSUP, i) {
            return vec![XNode::Cmd(i.to_string())];
        }

        if i == "set-scale" {
            return vec![if toks.len() == 3 {
                XNode::SetScale(toks[1].to_string(), toks[2].into())
            } else {
                xnode_from_perror!(InvalidInvocationError { cmd: "set-scale", usage: "PLG ['+'|'-']NUM" })
            }];
        }

        if i.len() != 1 || !scv_contains(&ALL_OPERATORS, i.chars().nth(0).unwrap()) {
            nodes.push(XNode::Calc('+', i.to_string(), XValue::Imm(prev_val)));
            toks.remove(0);
        }
    }

    let mut mode = ParserMode::Start;
    let mut st = '+';
    let mut clp: Option<&str> = None;

    for i in toks {
        match mode {
            ParserMode::Start => {
                if i.len() == 1 {
                    st = i.chars().nth(0).unwrap();
                    match st {
                        '+' | '-' | '*' | '/' | '%' => mode = ParserMode::Clp,
                        ':' | '=' => mode = ParserMode::ClpIX(st == ':'),
                        _ => {},
                    }
                }
                if mode == ParserMode::Start {
                    nodes.push(xnode_from_perror!(UnexpectedTokenError {expected: "'+'|'-'|'*'|'/'|':'|'='", got: i.to_string()}));
                }
            },
            ParserMode::Clp => {
                // expect calc plugin name
                clp = Some(i);
                mode = ParserMode::Num;
            },
            ParserMode::ClpIX(true) => {
                // expect calc plugin name
                nodes.push(XNode::CalcInv(i.to_string()));
                mode = ParserMode::Start;
            },
            ParserMode::ClpIX(false) => {
                // expect variable name
                nodes.push(XNode::Assign(i.to_string()));
                mode = ParserMode::Start;
            },
            ParserMode::Num => {
                nodes.push(XNode::Calc(st, clp.take().unwrap().to_string(), i.into()));
                mode = ParserMode::Start;
            },
        }
    }

    let xx: Option<&str> = match mode {
        ParserMode::Start => None,
        ParserMode::Clp | ParserMode::ClpIX(true) => Some("calc plugin name"),
        ParserMode::ClpIX(false) => Some("variable name"),
        ParserMode::Num => Some("number or variable name"),
    };

    if let Some(xx) = xx {
        nodes.push(xnode_from_perror!(UnexpectedEOLWhileError { expected: xx }));
    }

    nodes
}

fn eval_op(op: char, mut y: f64, x: f64) -> f64 {
    match op {
        '+' => y += x,
        '-' => y -= x,
        '*' => y *= x,
        '/' => y /= x,
        '%' => y = zxcalc_common::zx_modulo(y, x),
        _  => {},
    }
    y
}

fn getline() -> Option<String> {
    use termion::input::TermRead;
    use termion::event::Key;
    let mut curin = String::new();
    let stdin = std::io::stdin();

    for c in stdin.keys() {
        match c {
        Key::Char('\n') => break,
        Key::Ctrl('d') => if curin.empty() {
            return None;
        } else {
            break;
        },
        Key::Ctrl('c') => return if curin.empty() {
            None
        } else {
            Some(String::new())
        },
        Key::Char(c) => curin.push(c),
        _ => {},
        }
    }

    Some(curin)
}

fn getline_wprompt(prompt: &str) -> Option<String> {
    use std::io::Write;
    let mut stdout = std::io::stdout();
    write!(stdout, "{}", prompt).ok()?;
    stdout.flush().ok()?;
    getline()
}

fn main() {
    let value: f64 = 0.0;
    let mut cpm = cpm::CalcPluginManager::new();
    let mut vars: hashbrown::HashMap::<String, f64>::new();

    while let Some(curin) = getline_wprompt("zxcalc > ") {
        let mut got_error = false;
        let mut breakout = false;

        for i in parse_line(&curin, value) {
            let resolved_clp: Option<std::borrow::Cow<str>> = {
                match &i {
                    XNode::Calc(_, clp, _) | XNode::CalcInv(clp) | XNode::SetScale(clp, _) => {
                        if Some(clp2) = cpm.resolve(&clp) {
                            Some(clp2)
                        } else {
                            eprintln!("\tERROR: {}: unable to resolve plugin name", &clp);
                            got_error = true;
                            break;
                        }
                    },
                    _ => None,
                }
            };

            let resolved_val: Option<f64> = {
                match &i {
                    XNode::Calc(_, _, xval) | XNode::SetScale(_, xval) => {
                        match xval {
                            Imm(x) => Some(x),
                            Var(x) => if let Some(y) = vars.get(x) {
                                Some(y)
                            } else {
                                eprintln!("\tERROR: {}: unknown variable", &clp);
                                got_error = true;
                                break;
                            },
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
                },
                XNode::Cmd(cmd) => {
                    match &cmd[..] {
                        "exit" | "quit" => breakout = true,
                        "list-loaded-plugins" => cpm.list_loaded_plugins(),
                        "help" => {
                            println!(indoc!("
                              --COMMANDS--
                              \tquit\t\t\t\texit this program
                              \thelp\t\t\t\tprint this text
                              \tlist-loaded-plugins\t\tprint list of currently loaded plugins
                              \tset-scale PLG ['+'|'-']NUM\tset the scaling factor for the plugin (NUM < 0 --> division)

                                --SYNTAX--
                              \tThis program expects input lines not containing one of the
                              \tcommands above to have the following format:
                              \t\t(('+'|'-'|'*'|'/') PLG ['+'|'-']NUM|':' PLG|'=' VAR)*

                              "));
                        }
                        _ => eprintln!("\tERROR: unknown command {}\n", cmd),
                    }
                },
                XNode::SetScale(_, _) => {
                    // we don't want a "\t0\n" line after this
                    got_error = true;
                    let mut scval = resolved_val.take().unwrap();
                    if scval > 0.0 && scval < 1.0 {
                        scval = -(1.0 / scval);
                    }
                    cpm.set_scale(resolved_clp.take().unwrap(), scval);
                },
                XNode::Assign(varname) => {
                    vars.insert(varname, value);
                },
                XNode::Calc(op, _, _) => {
                    let clp = resolved_clp.take().unwrap();
                    let xval = resolved_val.take().unwrap();
                    if let Some(res) = cpm.calc(&clp, &xval) {
                        value = eval_op(op, value, res);
                    } else {
                        got_error = true;
                        eprintln!("\tERROR: {} {}: calc failed", &clp, &xval);
                    }
                },
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
