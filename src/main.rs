use rlox::{run_file, run_prompt};
use std::cmp::Ordering;
use std::{env, process};

fn main() {
    // env::set_var("RUST_BACKTRACE", "1");

    let args: Vec<String> = env::args().collect();

    match args.len().cmp(&2) {
        Ordering::Greater => {
            println!("Usage: rlox [script]");
            process::exit(64);
        }
        Ordering::Equal => run_file(&args[1]),
        _ => {
            run_prompt();
        }
    }
}
