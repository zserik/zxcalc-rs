pub type VarName = String;

#[derive(Clone, Debug, PartialEq)]
pub enum XValue {
    Var(VarName),
    Imm(f64),
}

pub type PluginName = String;

#[derive(Debug)]
pub enum XNode {
    Assign(VarName),
    Calc(char, PluginName, XValue),
    CalcInv(PluginName),
    Cmd(String),
    SetScale(PluginName, XValue),
    Error(failure::Error),
}

#[derive(Debug, Fail)]
pub enum XParseError {
    #[fail(display = "invalid invocation of '{}': USAGE: {} {}", cmd, cmd, usage)]
    InvalidInvocationError {
        cmd: &'static str,
        usage: &'static str,
    },

    #[fail(display = "expected one of {} instead of '{}'", expected, got)]
    UnexpectedTokenError { expected: &'static str, got: String },

    #[fail(display = "unexpected EOL while looking for {}", expected)]
    UnexpectedEOLWhileError { expected: &'static str },

    #[fail(display = "token no. {} is unexpected empty", pos)]
    UnexpectedEmptyError { pos: usize },
}

macro_rules! xnode_from_perror {
    ($et:ident { $($enam:ident : $val:expr),+ }) => {
        XNode::Error((XParseError::$et { $($enam: $val,)+ }).into())
    }
}

use crate::scv::{scv_contains, scv_create, SCV};
use crate::ssv::{ssv_contains, SSV};

lazy_static! {
    static ref CMDS2PASSUP: SSV = ssv_create!("exit", "help", "list-loaded-plugins", "quit");
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
pub enum ParserMode {
    Start,
    Clp,
    // ClpIX ? ClpInv : Assign
    ClpIX(bool),
    Num,
}

pub fn parse_line(line: &str, prev_val: f64) -> Vec<XNode> {
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
                xnode_from_perror!(InvalidInvocationError {
                    cmd: "set-scale",
                    usage: "PLG ['+'|'-']NUM"
                })
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
                        _ => {}
                    }
                }
                if mode == ParserMode::Start {
                    nodes.push(xnode_from_perror!(UnexpectedTokenError {
                        expected: "'+'|'-'|'*'|'/'|':'|'='",
                        got: i.to_string()
                    }));
                }
            }
            ParserMode::Clp => {
                // expect calc plugin name
                clp = Some(i);
                mode = ParserMode::Num;
            }
            ParserMode::ClpIX(true) => {
                // expect calc plugin name
                nodes.push(XNode::CalcInv(i.to_string()));
                mode = ParserMode::Start;
            }
            ParserMode::ClpIX(false) => {
                // expect variable name
                nodes.push(XNode::Assign(i.to_string()));
                mode = ParserMode::Start;
            }
            ParserMode::Num => {
                nodes.push(XNode::Calc(st, clp.take().unwrap().to_string(), i.into()));
                mode = ParserMode::Start;
            }
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
