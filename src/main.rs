pub mod expr;
pub mod macros;
pub mod matrix;
pub mod polynomial;
pub mod rational;
pub mod symbol;
pub mod term;

use crate::{expr::Expr, polynomial::Polynomial, rational::Rational, symbol::Symbol};
use std::{collections::HashMap, sync::Arc};

fn main() {
    let x = Arc::new(Symbol::new("x"));
    let y = Arc::new(Symbol::new("y"));
    let t = Arc::new(Symbol::new("t"));
    let mut symbols = HashMap::from([
        ("x".to_string(), Arc::clone(&x)),
        ("y".to_string(), Arc::clone(&y)),
        ("t".to_string(), Arc::clone(&t)),
    ]);

    let a: Expr<Rational> = Expr::from_str("x^3+2*x+1", &mut symbols).unwrap();
    let b = Expr::from_str("2*x+3", &mut symbols).unwrap();
    let a = Polynomial::try_from(&a).unwrap();
    let b = Polynomial::try_from(&b).unwrap();
    let (q, r) = a.divmod(&b, &x);
    println!("a={a}");
    println!("b={b}");
    println!("q={q}");
    println!("r={r}");

    println!();

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

    println!();

    let a: Expr<Rational> = Expr::from_str("x^3+1", &mut symbols).unwrap();
    let a = Polynomial::try_from(&a).unwrap();
    println!("a={a}");
    println!("a'={}", a.diff(&x));

    println!();

    let a: Expr<Rational> = Expr::from_str("x^3+3*x-1", &mut symbols).unwrap();
    let b = Expr::from_str("3*x^2+3", &mut symbols).unwrap();
    let a = Polynomial::try_from(&a).unwrap();
    let b = Polynomial::try_from(&b).unwrap();
    let s = a.resultant(&b, &x);
    println!("a={a}");
    println!("b={b}");
    println!("s={s}");

    println!();

    let a: Expr<Rational> = Expr::from_str("(t^2+1)*x-t", &mut symbols).unwrap();
    let b = Expr::from_str("(t^2+1)*y-2", &mut symbols).unwrap();
    let a = Polynomial::try_from(&a).unwrap();
    let b = Polynomial::try_from(&b).unwrap();
    let s = a.resultant(&b, &t);
    println!("a={a}");
    println!("b={b}");
    println!("s={s}");
}
