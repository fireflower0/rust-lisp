use regex::{Captures, Regex};
use std::rc::Rc;

use crate::types::RlErr::ErrString;
use crate::types::RlVal::{
    Bool,
    Int,
    List,
    Nil,
    Str,
    Sym,
    Vector
};
use crate::types::{error, hash_map, RlErr, RlRet, RlVal};

#[derive(Debug, Clone)]
struct Reader {
    tokens: Vec<String>,
    pos: usize,
}

impl Reader {
    fn next(&mut self) -> Result<String, RlErr> {
        self.pos = self.pos + 1;
        Ok(self
           .tokens
           .get(self.pos - 1)
           .ok_or(ErrString("underflow".to_string()))?
           .to_string())
    }
    fn peek(&self) -> Result<String, RlErr> {
        Ok(self
           .tokens
           .get(self.pos)
           .ok_or(ErrString("underflow".to_string()))?
           .to_string())
    }
}

fn tokenize(str: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r###"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"###
        )
        .unwrap();
    }
    let mut res = vec![];
    for cap in RE.captures_iter(str) {
        if cap[1].starts_with(";") {
            continue;
        }
        res.push(String::from(&cap[1]));
    }
    res
}

fn read_atom(rdr: &mut Reader) -> RlRet {
    lazy_static! {
        static ref INT_RE: Regex = Regex::new(r"^-?[0-9]+$").unwrap();
        static ref STR_RE: Regex = Regex::new(r#""(?:\\.|[^\\"])*""#).unwrap();
    }
    let token = rdr.next()?;
    match &token[..] {
        "nil" => Ok(Nil),
        "false" => Ok(Bool(false)),
        "true" => Ok(Bool(true)),
        _ => {
            if INT_RE.is_match(&token) {
                Ok(Int(token.parse().unwrap()))
            } else {
                Ok(Sym(token.to_string()))
            }
        }
    }
}

fn read_seq(rdr: &mut Reader, end: &str) -> RlRet {
    let mut seq: Vec<RlVal> = vec![];
    rdr.next()?;
    loop {
        let token = match rdr.peek() {
            Ok(t) => t,
            Err(_) => return error(&format!("expected '{}', got EOF", end)),
        };
        if token == end {
            break;
        }
        seq.push(read_form(rdr)?)
    }
    let _ = rdr.next();
    match end {
        ")" => Ok(list!(seq)),
        "]" => Ok(vector!(seq)),
        "}" => hash_map(seq),
        _ => error("read_seq unknown end value"),
    }
}

fn read_form(rdr: &mut Reader) -> RlRet {
    let token = rdr.peek()?;
    match &token[..] {
        "'" => {
            let _ = rdr.next();
            Ok(list![Sym("quote".to_string()), read_form(rdr)?])
        }
        "`" => {
            let _ = rdr.next();
            Ok(list![Sym("quasiquote".to_string()), read_form(rdr)?])
        }
        "~" => {
            let _ = rdr.next();
            Ok(list![Sym("unquote".to_string()), read_form(rdr)?])
        }
        "~@" => {
            let _ = rdr.next();
            Ok(list![Sym("splice-unquote".to_string()), read_form(rdr)?])
        }
        "^" => {
            let _ = rdr.next();
            let meta = read_form(rdr)?;
            Ok(list![Sym("with-meta".to_string()), read_form(rdr)?, meta])
        }
        "@" => {
            let _ = rdr.next();
            Ok(list![Sym("deref".to_string()), read_form(rdr)?])
        }
        ")" => error("unexpected ')'"),
        "(" => read_seq(rdr, ")"),
        "]" => error("unexpected ']'"),
        "[" => read_seq(rdr, "]"),
        _ => read_atom(rdr),
    }
}

pub fn read_str(str: String) -> RlRet {
    let tokens = tokenize(&str);
    if tokens.len() == 0 {
        return error("no input");
    }
    read_form(&mut Reader {
        pos: 0,
        tokens: tokens,
    })
}
