extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

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
