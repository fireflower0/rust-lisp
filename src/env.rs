use std::cell::RefCell;
use std::rc::Rc;
use fnv::FnvHashMap;

use crate::types::RlErr::ErrString;
use crate::types::RlVal::{List, Nil, Sym, Vector};
use crate::types::{error, RlErr, RlRet, RlVal};

#[derive(Debug)]
pub struct EnvStruct {
    data: RefCell<FnvHashMap<String, RlVal>>,
    pub outer: Option<Env>,
}

pub type Env = Rc<EnvStruct>;

pub fn env_new(outer: Option<Env>) -> Env {
    Rc::new(EnvStruct {
        data: RefCell::new(FnvHashMap::default()),
        outer: outer,
    })
}

pub fn env_bind(outer: Option<Env>, mbinds: RlVal, exprs: Vec<RlVal>) -> Result<Env, RlErr> {
    let env = env_new(outer);
    match mbinds {
        List(binds, _) | Vector(binds, _) => {
            for (i, b) in binds.iter().enumerate() {
                match b {
                    Sym(s) if s == "&" => {
                        env_set(&env, binds[i + 1].clone(), list!(exprs[i..].to_vec()))?;
                        break;
                    }
                    _ => {
                        env_set(&env, b.clone(), exprs[i].clone())?;
                    }
                }
            }
            Ok(env)
        }
        _ => Err(ErrString("env_bind binds not List/Vector".to_string())),
    }
}

pub fn env_find(env: &Env, key: &str) -> Option<Env> {
    match (env.data.borrow().contains_key(key), env.outer.clone()) {
        (true, _) => Some(env.clone()),
        (false, Some(o)) => env_find(&o, key),
        _ => None,
    }
}

pub fn env_get(env: &Env, key: &RlVal) -> RlRet {
    match key {
        Sym(ref s) => match env_find(env, s) {
            Some(e) => Ok(e
                          .data
                          .borrow()
                          .get(s)
                          .ok_or(ErrString(format!("'{}( not found", s)))?
                          .clone()),
            _ => error(&format!("'{}' not found", s)),
        }
        _ => error("Env.get called with non-Str"),
    }
}

pub fn env_set(env: &Env, key: RlVal, val: RlVal) -> RlRet {
    match key {
        Sym(ref s) => {
            env.data.borrow_mut().insert(s.to_string(), val.clone());
            Ok(val)
        }
        _ => error("Env.set called with non-Str"),
    }
}

pub fn env_sets(env: &Env, key: &str, val: RlVal) {
    env.data.borrow_mut().insert(key.to_string(), val);
}
