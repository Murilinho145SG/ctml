use std::{env, fs};

use ctml_compiler::Lexer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/hello.ctml".to_owned());
    let source = fs::read_to_string(path)?;

    for token in Lexer::new(&source).tokenize()? {
        println!("{token:?}");
    }

    Ok(())
}
