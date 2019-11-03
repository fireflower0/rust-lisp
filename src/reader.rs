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
