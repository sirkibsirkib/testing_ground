

const HISTORY_PATH : &'static str = "history.txt";

use ::rustyline::error::ReadlineError;
use ::rustyline::Editor;

pub fn go() {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if let Err(_) = rl.load_history(HISTORY_PATH) {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                if line == "clear" {
                    rl.clear_history();
                } else {
                    rl.add_history_entry(&line);
                    println!("Line: {}", line);
                }
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history(HISTORY_PATH).unwrap();
}
