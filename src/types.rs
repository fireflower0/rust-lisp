use std::cell::RefCell;
use std::rc::Rc;
use fnv::FnvHashMap;
use itertools::Itertools;

use crate::env::{env_bind, Env};
use crate::types::RlErr::{ErrRlVal, ErrString};
use crate::types::RlVal::{
    Atom,
    Bool,
    Func,
    Hash,
    Int,
    List,
    RlFunc,
    Nil,
    Str,
    Sym,
    Vector
};

#[derive(Debug, Clone)]
pub enum RlVal {
    Nil,
    Bool(bool),
    Int(i64),
    Str(String),
    Sym(String),
    List(Rc<Vec<RlVal>>, Rc<RlVal>),
    Vector(Rc<Vec<RlVal>>, Rc<RlVal>),
    Hash(Rc<FnvHashMap<String, RlVal>>, Rc<RlVal>),
    Func(fn(RlArgs) -> RlRet, Rc<RlVal>),
    RlFunc {
        eval: fn(ast: RlVal, env: Env) -> RlRet,
        ast: Rc<RlVal>,
        env: Env,
        params: Rc<RlVal>,
        is_macro: bool,
        meta: Rc<RlVal>,
    },
    Atom(Rc<RefCell<RlVal>>),
}

#[derive(Debug)]
pub enum RlErr {
    ErrString(String),
    ErrRlVal(RlVal),
}

pub type RlArgs = Vec<RlVal>;
pub type RlRet  = Result<RlVal, RlErr>;

macro_rules! list {
    ($seq:expr) => {{
        List(Rc::new($seq), Rc::new(Nil))
    }};
    [$($args:expr), *] => {{
        let v: Vec<RlVal> = vec![$($args), *];
        List(Rc::new(v), Rc::new(Nil))
    }}
}

pub fn error(s: &str) -> RlRet {
    Err(ErrString(s.to_string()))
}
