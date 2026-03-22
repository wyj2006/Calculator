pub mod expr;
pub mod macros;
pub mod polynomial;
pub mod rational;
pub mod symbol;
pub mod term;

use crate::{expr::Expr, polynomial::Polynomial, rational::Rational, symbol::Symbol};
use std::{collections::HashMap, sync::Arc};

fn main() {
    let x = Arc::new(Symbol::new("x"));
    let y = Arc::new(Symbol::new("y"));
    let mut symbols = HashMap::from([
        ("x".to_string(), Arc::clone(&x)),
        ("y".to_string(), Arc::clone(&y)),
    ]);
    let a: Expr<Rational> = Expr::from_str("2*y^3-y^2+x^2*y", &mut symbols).unwrap();
    let b = Expr::from_str("x*y^2+1", &mut symbols).unwrap();
    let a = Polynomial::try_from(&a).unwrap();
    let b = Polynomial::try_from(&b).unwrap();
    let (q, r, s) = a.pseudo_divmod(&b, &y);
    println!("a={a}");
    println!("b={b}");
    println!("q={q}");
    println!("r={r}");
    println!("s={s}");
}
