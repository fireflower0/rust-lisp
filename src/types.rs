use fnv::FnvHashMap;
use itertools::Itertools;
use std::cell::RefCell;
use std::rc::Rc;

use crate::env::{env_bind, Env};
use crate::types::RlErr::{ErrRlVal, ErrString};
use crate::types::RlVal::{Atom, Bool, Func, Hash, Int, List, Nil, RlFunc, Str, Sym, Vector};

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
pub type RlRet = Result<RlVal, RlErr>;

// Macro

macro_rules! list {
    ($seq:expr) => {{
        List(Rc::new($seq), Rc::new(Nil))
    }};
    [$($args:expr), *] => {{
        let v: Vec<RlVal> = vec![$($args), *];
        List(Rc::new(v), Rc::new(Nil))
    }}
}

macro_rules! vector {
    ($seq:expr) => {{
        Vector(Rc::new($seq), Rc::new(Nil))
    }};
    [$($args:expr), *] => {{
        let v: Vec<RlVal> = vec![$($args), *];
        Vector(Rc::new(v), Rc::new(Nil))
    }}
}

// type utility functions

pub fn error(s: &str) -> RlRet {
    Err(ErrString(s.to_string()))
}

pub fn format_error(e: RlErr) -> String {
    match e {
        ErrString(s) => s.clone(),
        ErrRlVal(mv) => mv.pr_str(true),
    }
}

pub fn atom(rv: &RlVal) -> RlVal {
    Atom(Rc::new(RefCell::new(rv.clone())))
}

// Functions

impl RlVal {
    pub fn keyword(&self) -> RlRet {
        match self {
            Str(s) if s.starts_with("\u{29e}") => Ok(Str(s.to_string())),
            Str(s) => Ok(Str(format!("\u{29e}{}", s))),
            _ => error("invalid type for keyword"),
        }
    }

    pub fn empty_q(&self) -> RlRet {
        match self {
            List(l, _) | Vector(l, _) => Ok(Bool(l.len() == 0)),
            Nil => Ok(Bool(true)),
            _ => error("invalid type for empty?"),
        }
    }

    pub fn count(&self) -> RlRet {
        match self {
            List(l, _) | Vector(l, _) => Ok(Int(l.len() as i64)),
            Nil => Ok(Int(0)),
            _ => error("invalid type for count"),
        }
    }

    pub fn apply(&self, args: RlArgs) -> RlRet {
        match *self {
            Func(f, _) => f(args),
            RlFunc {
                eval,
                ref ast,
                ref env,
                ref params,
                ..
            } => {
                let a = &**ast;
                let p = &**params;
                let fn_env = env_bind(Some(env.clone()), p.clone(), args)?;
                Ok(eval(a.clone(), fn_env)?)
            }
            _ => error("attempt to call non-function"),
        }
    }

    pub fn keyword_q(&self) -> bool {
        match self {
            Str(s) if s.starts_with("\u{29e}") => true,
            _ => false,
        }
    }

    pub fn deref(&self) -> RlRet {
        match self {
            Atom(a) => Ok(a.borrow().clone()),
            _ => error("attempt to defef a non-Atom"),
        }
    }

    pub fn reset_bang(&self, new: &RlVal) -> RlRet {
        match self {
            Atom(a) => {
                *a.borrow_mut() = new.clone();
                Ok(new.clone())
            }
            _ => error("attempt to reset! a non-Atom"),
        }
    }

    pub fn swap_bang(&self, args: &RlArgs) -> RlRet {
        match self {
            Atom(a) => {
                let f = &args[0];
                let mut fargs = args[1..].to_vec();
                fargs.insert(0, a.borrow().clone());
                *a.borrow_mut() = f.apply(fargs)?;
                Ok(a.borrow().clone())
            }
            _ => error("attempt to swap! a non-Atom"),
        }
    }

    pub fn get_meta(&self) -> RlRet {
        match self {
            List(_, meta) | Vector(_, meta) | Hash(_, meta) => Ok((&**meta).clone()),
            Func(_, meta) => Ok((&**meta).clone()),
            RlFunc { meta, .. } => Ok((&**meta).clone()),
            _ => error("meta not supported by type"),
        }
    }

    pub fn with_meta(&mut self, new_meta: &RlVal) -> RlRet {
        match self {
            List(_, ref mut meta)
                | Vector(_, ref mut meta)
                | Hash(_, ref mut meta)
                | Func(_, ref mut meta)
                | RlFunc { ref mut meta, .. } => {
                    *meta = Rc::new((&*new_meta).clone());
                }
            _ => return error("with-meta not supported by type"),
        };
        Ok(self.clone())
    }
}

impl PartialEq for RlVal {
    fn eq(&self, other: &RlVal) -> bool {
        match (self, other) {
            (Nil, Nil) => true,
            (Bool(ref a), Bool(ref b)) => a == b,
            (Int(ref a), Int(ref b)) => a == b,
            (Str(ref a), Str(ref b)) => a == b,
            (Sym(ref a), Sym(ref b)) => a == b,
            (List(ref a, _), List(ref b, _))
                | (Vector(ref a, _), Vector(ref b, _))
                | (List(ref a, _), Vector(ref b, _))
                | (Vector(ref a, _), List(ref b, _)) => a == b,
            (Hash(ref a, _), Hash(ref b, _)) => a == b,
            (RlFunc { .. }, RlFunc { .. }) => false,
            _ => false,
        }
    }
}

pub fn func(f: fn(RlArgs) -> RlRet) -> RlVal {
    Func(f, Rc::new(Nil))
}

pub fn _assoc(mut hm: FnvHashMap<String, RlVal>, kvs: RlArgs) -> RlRet {
    if kvs.len() % 2 != 0 {
        return error("odd number of elements");
    }
    for (k, v) in kvs.iter().tuples() {
        match k {
            Str(s) => {
                hm.insert(s.to_string(), v.clone());
            }
            _ => return error("key is not string"),
        }
    }
    Ok(Hash(Rc::new(hm), Rc::new(Nil)))
}

pub fn _dissoc(mut hm: FnvHashMap<String, RlVal>, kvs: RlArgs) -> RlRet {
    if kvs.len() % 2 != 0 {
        return error("odd number of elements");
    }
    for (k, v) in kvs.iter().tuples() {
        match k {
            Str(s) => {
                hm.insert(s.to_string(), v.clone());
            }
            _ => return error("key is not string"),
        }
    }
    Ok(Hash(Rc::new(hm), Rc::new(Nil)))
}

pub fn hash_map(kvs: RlArgs) -> RlRet {
    let hm: FnvHashMap<String, RlVal> = FnvHashMap::default();
    _assoc(hm, kvs)
}
