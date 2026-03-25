pub mod expr;
pub mod interpreter;
pub mod macros;
pub mod matrix;
pub mod polynomial;
pub mod print;
pub mod rational;
pub mod symbol;
pub mod term;

use crate::{expr::Expr, interpreter::Interpreter, rational::Rational, symbol::Symbol};
use std::{
    collections::HashMap,
    io::{Write, stdin, stdout},
    sync::Arc,
};

fn main() {
    let mut interpreter: Interpreter<Rational> = Interpreter::new(HashMap::from([(
        "pi".to_string(),
        Expr::from(&Arc::new(Symbol::new("π"))),
    )]));
    let mut inputs = vec![];

    loop {
        print!("In [{}]: ", inputs.len() + 1);
        stdout().flush().unwrap();

        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        let buf = buf.trim().to_string();

        match interpreter.execute(&buf) {
            Ok(_) => {}
            Err(e) => println!("error: {e}"),
        }

        inputs.push(buf);
    }
}
