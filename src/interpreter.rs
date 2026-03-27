use crate::{expr::Expr, polynomial::Polynomial, print::Print, symbol::Symbol};
use anyhow::{Result, anyhow};
use bigdecimal::Pow;
use num::{BigUint, One, Zero};
use pest::{
    Parser,
    pratt_parser::{Assoc, Op, PrattParser},
};
use pest_derive::Parser;
use std::{
    collections::HashMap,
    error::Error,
    fmt::Display,
    ops::{Div, Neg, Sub},
    str::FromStr,
    sync::{Arc, LazyLock},
};

pub static PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
        .op(Op::infix(Rule::pow, Assoc::Right))
});

#[derive(Debug, Parser)]
#[grammar = "parser.pest"]
pub struct Interpreter<C> {
    pub vars: HashMap<String, Expr<C>>,
}

impl<C> Interpreter<C> {
    pub fn new(vars: HashMap<String, Expr<C>>) -> Interpreter<C> {
        Interpreter { vars }
    }
}

impl<C> Interpreter<C>
where
    for<'a> C: FromStr
        + Zero
        + Clone
        + One
        + PartialEq
        + Neg<Output = C>
        + Div<Output = C>
        + Display
        + From<BigUint>
        + Sub<Output = C>
        + From<BigUint>
        + Pow<Expr<C>, Output = Expr<C>>,
    <C as FromStr>::Err: Error,
    BigUint: TryFrom<C>,
{
    pub fn execute(&mut self, input: &str) -> Result<()> {
        for rule in Self::parse(Rule::statement, input)?
            .next()
            .unwrap()
            .into_inner()
        {
            match rule.as_rule() {
                Rule::expr => {
                    let expr = Expr::<C>::from_str(rule.as_str(), &mut self.vars)?;
                    expr.simplify().print();
                }
                Rule::assign => {
                    let mut name = "";
                    let mut value = Expr::zero();
                    for rule in rule.into_inner() {
                        match rule.as_rule() {
                            Rule::identifier => name = rule.as_str(),
                            Rule::expr => value = Expr::from_str(rule.as_str(), &mut self.vars)?,
                            _ => unreachable!(),
                        }
                    }
                    self.vars.insert(name.to_string(), value);
                }
                Rule::EOI => {}
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}

impl<C> Expr<C>
where
    for<'a> C: FromStr
        + Zero
        + Clone
        + One
        + PartialEq
        + Neg<Output = C>
        + Div<Output = C>
        + From<BigUint>
        + Sub<Output = C>
        + From<BigUint>
        + Pow<Expr<C>, Output = Expr<C>>,
    <C as FromStr>::Err: Error,
    BigUint: TryFrom<C>,
{
    pub fn from_str(input: &str, vars: &mut HashMap<String, Expr<C>>) -> Result<Self> {
        PRATT_PARSER
            .map_primary(|primary| {
                Ok(match primary.as_rule() {
                    Rule::symbol => {
                        if let Some(t) = vars.get(primary.as_str()) {
                            t.clone()
                        } else {
                            let sym = Expr::from(&Arc::new(Symbol::new(primary.as_str())));
                            vars.insert(primary.as_str().to_string(), sym.clone());
                            sym
                        }
                    }
                    Rule::number => Expr::Const(match C::from_str(primary.as_str()) {
                        Ok(t) => t,
                        Err(e) => Err(anyhow!("{}", e.to_string()))?,
                    }),
                    Rule::expr => Expr::from_str(primary.as_str(), vars)?,
                    Rule::call => {
                        let mut name = "???";
                        let mut args = vec![];
                        for rule in primary.into_inner() {
                            match rule.as_rule() {
                                Rule::identifier => name = rule.as_str(),
                                Rule::expr => args.push(Self::from_str(rule.as_str(), vars)?),
                                _ => unreachable!(),
                            }
                        }

                        match name {
                            "ploy_div" | "poly_mod" | "pseudo_div" | "pseudo_mod" => {
                                let a = Polynomial::<C>::try_from(
                                    args.get(0).ok_or(anyhow!("too few arguments"))?,
                                )
                                .map_err(|e| anyhow!(e))?;

                                let b = Polynomial::<C>::try_from(
                                    args.get(1).ok_or(anyhow!("too few arguments"))?,
                                )
                                .map_err(|e| anyhow!(e))?;

                                match args.get(2).ok_or(anyhow!("too few arguments"))? {
                                    Expr::Symbol(sym) => {
                                        let (q, r) = match name {
                                            "ploy_div" | "ploy_mod" => a.divmod(&b, sym),
                                            "pseudo_div" | "pseudo_mod" => {
                                                let t = a.pseudo_divmod(&b, sym);
                                                (t.0, t.1)
                                            }
                                            _ => unreachable!(),
                                        };

                                        match name {
                                            "ploy_div" | "pseudo_div" => Expr::from(&q),
                                            "poly_mod" | "pseudo_mod" => Expr::from(&r),
                                            _ => unreachable!(),
                                        }
                                    }
                                    _ => Err(anyhow!("must provide a symbol"))?,
                                }
                            }
                            "diff" => {
                                let a = Polynomial::<C>::try_from(
                                    args.get(0).ok_or(anyhow!("too few arguments"))?,
                                )
                                .map_err(|e| anyhow!(e))?;

                                match args.get(1).ok_or(anyhow!("too few arguments"))? {
                                    Expr::Symbol(sym) => Expr::from(&a.diff(sym)),
                                    _ => Err(anyhow!("must provide a symbol"))?,
                                }
                            }
                            "resultant" | "gcd" => {
                                let a = Polynomial::<C>::try_from(
                                    args.get(0).ok_or(anyhow!("too few arguments"))?,
                                )
                                .map_err(|e| anyhow!(e))?;

                                let b = Polynomial::<C>::try_from(
                                    args.get(1).ok_or(anyhow!("too few arguments"))?,
                                )
                                .map_err(|e| anyhow!(e))?;

                                match args.get(2).ok_or(anyhow!("too few arguments"))? {
                                    Expr::Symbol(sym) => match name {
                                        "resultant" => a.resultant(&b, sym),
                                        "gcd" => Expr::from(&a.gcd(&b, sym).0),
                                        _ => unreachable!(),
                                    },
                                    _ => Err(anyhow!("must provide a symbol"))?,
                                }
                            }
                            "total_degree" => {
                                let a = Polynomial::<C>::try_from(
                                    args.get(0).ok_or(anyhow!("too few arguments"))?,
                                )
                                .map_err(|e| anyhow!(e))?;

                                Expr::Const(C::from(a.total_degree()))
                            }
                            "degree" | "leading_coef" => {
                                let a = Polynomial::<C>::try_from(
                                    args.get(0).ok_or(anyhow!("too few arguments"))?,
                                )
                                .map_err(|e| anyhow!(e))?;

                                match args.get(1).ok_or(anyhow!("too few arguments"))? {
                                    Expr::Symbol(sym) => match name {
                                        "degree" => Expr::Const(C::from(a.degree(sym))),
                                        "leading_coef" => a.leading_coef(sym),
                                        _ => unreachable!(),
                                    },
                                    _ => Err(anyhow!("must provide a symbol"))?,
                                }
                            }
                            "coef" => {
                                let a = Polynomial::<C>::try_from(
                                    args.get(0).ok_or(anyhow!("too few arguments"))?,
                                )
                                .map_err(|e| anyhow!(e))?;

                                match (
                                    args.get(1).ok_or(anyhow!("too few arguments"))?,
                                    args.get(2).ok_or(anyhow!("too few arguments"))?,
                                ) {
                                    (Expr::Symbol(sym), Expr::Const(deg)) => a.coef(
                                        sym,
                                        BigUint::try_from(deg.clone()).map_err(|_| {
                                            anyhow!("degree must be an positive integer constant")
                                        })?,
                                    ),
                                    _ => Err(anyhow!(
                                        "must provide a symbol and an positive integer constant"
                                    ))?,
                                }
                            }
                            "expand" => {
                                let a = args.get(0).ok_or(anyhow!("too few arguments"))?;
                                a.expand()
                            }
                            _ => Err(anyhow!("unknown function: {name}"))?,
                        }
                    }
                    _ => unreachable!(),
                })
            })
            .map_infix(|lhs, op, rhs| {
                Ok(match op.as_rule() {
                    Rule::add => lhs? + rhs?,
                    Rule::sub => lhs? - rhs?,
                    Rule::mul => lhs? * rhs?,
                    Rule::div => lhs? / rhs?,
                    //TODO 或许有别的方式
                    Rule::pow => {
                        Expr::Pow(Box::new(lhs?.simplify()), Box::new(rhs?.simplify())).simplify()
                    }
                    _ => unreachable!(),
                })
            })
            .parse(
                Interpreter::<C>::parse(Rule::expr, input)?
                    .next()
                    .unwrap()
                    .into_inner(),
            )
    }
}
