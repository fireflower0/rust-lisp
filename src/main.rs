#[macro_use]
extern crate lazy_static;
extern crate fnv;
extern crate itertools;
extern crate regex;

extern crate rustyline;
use rustyline::error::ReadlineError;
use rustyline::Editor;

#[macro_use]
#[allow(dead_code)]
mod types;

fn main() {
    let mut r1 = Editor::<()>::new();

    if r1.load_history(".history").is_err() {
        eprintln!("No previous history.");
    }

    loop {
        let readline = r1.readline("user> ");
        match readline {
            Ok(line) => {
                r1.add_history_entry(&line);
                r1.save_history(".history").unwrap();
                if line.len() > 0 {
                    println!("{}", line);
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
