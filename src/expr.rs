use crate::{forward_impl_binop, polynomial::Polynomial, symbol::Symbol, term::Term};
use num::{BigUint, One, Zero, pow::Pow};
use pest::{
    Parser,
    error::Error,
    pratt_parser::{Assoc, Op, PrattParser},
};
use pest_derive::Parser;
use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
    sync::{Arc, LazyLock},
};

#[derive(Debug, Clone, Parser)]
#[grammar = "parser.pest"]
pub enum Expr<C> {
    Const(C),
    Symbol(Arc<Symbol>),
    Add(Vec<Expr<C>>),
    Mul(Vec<Expr<C>>),
    Pow(Box<Expr<C>>, Box<Expr<C>>),
}

impl<C> PartialEq for Expr<C>
where
    C: Zero + Clone + One + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (self.flatten(), other.flatten()) {
            (Expr::Const(a), Expr::Const(b)) => a == b,
            (Expr::Symbol(a), Expr::Symbol(b)) => a == b,
            (Expr::Add(a), Expr::Add(b)) | (Expr::Mul(a), Expr::Mul(b)) => {
                //如果 x==y, 那么 hash(x)==hash(y)
                let mut t = a;
                for i in b {
                    if let Some(k) = t.iter().position(|x| *x == i) {
                        t.remove(k);
                    } else {
                        return false;
                    }
                }
                true
            }
            (Expr::Pow(a1, b1), Expr::Pow(a2, b2)) => a1 == a2 && b1 == b2,
            _ => false,
        }
    }
}

impl<C> Display for Expr<C>
where
    C: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Const(t) => write!(f, "{t}")?,
            Expr::Symbol(t) => write!(f, "{t}")?,
            Expr::Add(t) => {
                for (i, v) in t.iter().enumerate() {
                    let s = v.to_string();
                    if i > 0 && !s.starts_with("-") {
                        write!(f, "+")?;
                    }
                    write!(f, "{s}")?;
                }
            }
            Expr::Mul(t) => {
                for (i, v) in t.iter().enumerate() {
                    let s = v.to_string();
                    if i > 0 {
                        write!(f, "*")?;
                    }
                    match v {
                        Expr::Add(_) => write!(f, "({s})")?,
                        _ => write!(f, "{s}")?,
                    }
                }
            }
            Expr::Pow(a, b) => {
                match **a {
                    Expr::Add(_) | Expr::Mul(_) => write!(f, "({a})")?,
                    _ => write!(f, "{a}")?,
                }
                write!(f, "^")?;
                match **b {
                    Expr::Add(_) | Expr::Mul(_) => write!(f, "({b})")?,
                    _ => write!(f, "{b}")?,
                }
            }
        }
        Ok(())
    }
}

impl<C> Zero for Expr<C>
where
    C: Zero + One + PartialEq + Clone,
{
    fn is_zero(&self) -> bool {
        match self {
            Expr::Const(t) => t.is_zero(),
            Expr::Add(t) => t.iter().all(|x| x.is_zero()),
            Expr::Mul(t) => t.iter().any(|x| x.is_zero()),
            Expr::Pow(a, _) => a.is_zero(),
            _ => false,
        }
    }

    fn zero() -> Self {
        Expr::Const(C::zero())
    }
}

impl<C> One for Expr<C>
where
    C: One + PartialEq + Zero + Clone,
{
    fn one() -> Self {
        Expr::Const(C::one())
    }

    fn is_one(&self) -> bool {
        match self {
            Expr::Const(t) => t.is_one(),
            Expr::Add(t) => {
                let mut a = 0;
                for i in t {
                    //可能是任意数
                    if !i.is_one() && !i.is_zero() {
                        return false;
                    }
                    a += i.is_one() as u32;
                }
                a == 1
            }
            Expr::Mul(t) => {
                let mut a = 1;
                for i in t {
                    //可能是任意数
                    if !i.is_one() && !i.is_zero() {
                        return false;
                    }
                    a *= i.is_one() as u32;
                }
                a == 1
            }
            Expr::Pow(_, b) => b.is_zero(),
            _ => false,
        }
    }
}

impl<C> Expr<C>
where
    C: Zero + One + PartialEq + Clone,
{
    pub fn flatten(&self) -> Expr<C> {
        match self {
            t if t.is_zero() => Expr::zero(),
            t if t.is_one() => Expr::one(),
            Expr::Add(t) => {
                let mut a = vec![];
                for i in t {
                    match i.flatten() {
                        t if t.is_zero() => {}
                        Expr::Add(t) => a.extend(t),
                        t => a.push(t),
                    }
                }
                if a.len() == 0 {
                    Expr::zero()
                } else if a.len() == 1 {
                    a.remove(0)
                } else {
                    Expr::Add(a)
                }
            }
            Expr::Mul(t) => {
                let mut a = vec![];
                for i in t {
                    match i.flatten() {
                        t if t.is_one() => {}
                        Expr::Mul(t) => a.extend(t),
                        t => a.push(t),
                    }
                }
                if a.len() == 0 {
                    Expr::zero()
                } else if a.len() == 1 {
                    a.remove(0)
                } else {
                    Expr::Mul(a)
                }
            }
            Expr::Pow(a, b) if b.is_one() => a.flatten(),
            _ => self.clone(),
        }
    }
}

impl<C> Add<&Expr<C>> for &Expr<C>
where
    C: Zero + Clone + One + PartialEq,
{
    type Output = Expr<C>;

    fn add(self, rhs: &Expr<C>) -> Self::Output {
        match (self.flatten(), rhs.flatten()) {
            (Expr::Const(a), Expr::Const(b)) => Expr::Const(a + b),
            (Expr::Add(mut a), b @ Expr::Const(_)) | (b @ Expr::Const(_), Expr::Add(mut a)) => {
                let mut t = Vec::new();
                //与加法算式中的常量相加
                while a.len() > 0 {
                    let x = a.remove(0);
                    if matches!(x, Expr::Const(_)) {
                        t.push(x + b);
                        break;
                    } else {
                        t.push(x);
                    }
                }
                //剩余部分
                while a.len() > 0 {
                    t.push(a.remove(0));
                }
                Expr::Add(t).flatten()
            }
            (a, b) => Expr::Add(vec![a, b]).flatten(),
        }
    }
}

forward_impl_binop!(impl<C> Add for Expr<C>, add, where C: Zero + Clone + One + PartialEq);

impl<C> Sub<&Expr<C>> for &Expr<C>
where
    C: Zero + Clone + One + PartialEq + Neg<Output = C>,
{
    type Output = Expr<C>;

    fn sub(self, rhs: &Expr<C>) -> Self::Output {
        match (self.flatten(), rhs.flatten()) {
            (a, b) if a == b => Expr::zero(),
            (a, b) => a + b * Expr::Const(-C::one()),
        }
    }
}

forward_impl_binop!(impl<C> Sub for Expr<C>, sub, where C: Zero + Clone + One + PartialEq + Neg<Output = C>);

impl<C> Mul<&Expr<C>> for &Expr<C>
where
    C: Zero + Clone + One + PartialEq,
{
    type Output = Expr<C>;

    fn mul(self, rhs: &Expr<C>) -> Self::Output {
        match (self.flatten(), rhs.flatten()) {
            (Expr::Const(a), Expr::Const(b)) => Expr::Const(a * b),
            (Expr::Mul(mut a), b @ Expr::Const(_)) | (b @ Expr::Const(_), Expr::Mul(mut a)) => {
                let mut t = Vec::new();
                //与乘法算式中的常量相乘
                while a.len() > 0 {
                    let x = a.remove(0);
                    if matches!(x, Expr::Const(_)) {
                        t.push(x * b);
                        break;
                    } else {
                        t.push(x);
                    }
                }
                //剩余部分
                while a.len() > 0 {
                    t.push(a.remove(0));
                }
                Expr::Mul(t).flatten()
            }
            (a, b) => Expr::Mul(vec![a, b]).flatten(),
        }
    }
}

forward_impl_binop!(impl<C> Mul for Expr<C>,mul,where C: Zero + Clone + One + PartialEq);

impl<C> Div<&Expr<C>> for &Expr<C>
where
    C: Div<Output = C> + Zero + Clone + One + PartialEq + Neg<Output = C>,
{
    type Output = Expr<C>;

    fn div(self, rhs: &Expr<C>) -> Self::Output {
        match (self.flatten(), rhs.flatten()) {
            (Expr::Const(a), Expr::Const(b)) => Expr::Const(a / b),
            (a, b) if a == b => Expr::one(),
            (a, b) => a * Expr::Pow(Box::new(b), Box::new(Expr::Const(-C::one()))),
        }
    }
}

forward_impl_binop!(impl<C> Div for Expr<C>,div,where C: Div<Output = C> + Zero + Clone + One + PartialEq + Neg<Output = C>);

impl<CL, CR> Pow<&Expr<CR>> for &Expr<CL>
where
    CL: Zero + Clone + One + PartialEq + Pow<CR, Output = CL>,
    CR: Zero + Clone + One + PartialEq,
    Expr<CL>: From<Expr<CR>>,
{
    type Output = Expr<CL>;

    fn pow(self, rhs: &Expr<CR>) -> Self::Output {
        match (self.flatten(), rhs.flatten()) {
            (Expr::Const(a), Expr::Const(b)) => Expr::Const(a.pow(b)),
            (a, b) => Expr::Pow(Box::new(a), Box::new(b.into())).flatten(),
        }
    }
}

forward_impl_binop!(
    impl<CL, CR> Pow<Expr<CR>> for Expr<CL>,
    pow,
    where
        CL: Zero + Clone + One + PartialEq + Pow<CR, Output = CL>,
        CR: Zero + Clone + One + PartialEq,
        Expr<CL>: From<Expr<CR>>,
);

impl<C> Neg for &Expr<C>
where
    C: Neg<Output = C> + Clone + Zero + One + PartialEq,
{
    type Output = Expr<C>;

    fn neg(self) -> Self::Output {
        match self.flatten() {
            Expr::Const(t) => Expr::Const(-t),
            Expr::Add(t) => Expr::Add(t.iter().map(|x| x * -Expr::one()).collect()).flatten(),
            t => -Expr::one() * t,
        }
    }
}

impl<C> Neg for Expr<C>
where
    C: Neg<Output = C> + Clone + Zero + One + PartialEq,
{
    type Output = Expr<C>;

    fn neg(self) -> Self::Output {
        (&self).neg()
    }
}

impl<CI, CO> From<&Expr<CI>> for Expr<CO>
where
    CI: Clone,
    CO: From<CI>,
{
    fn from(value: &Expr<CI>) -> Self {
        match value {
            Expr::Const(t) => Expr::Const(CO::from(t.clone())),
            Expr::Symbol(t) => Expr::Symbol(Arc::clone(t)),
            Expr::Add(t) => Expr::Add(t.iter().map(|x| Expr::from(x)).collect()),
            Expr::Mul(t) => Expr::Mul(t.iter().map(|x| Expr::from(x)).collect()),
            Expr::Pow(a, b) => Expr::Pow(Box::new(Expr::from(&**a)), Box::new(Expr::from(&**b))),
        }
    }
}

impl<C> From<&Arc<Symbol>> for Expr<C> {
    fn from(value: &Arc<Symbol>) -> Self {
        Expr::Symbol(Arc::clone(value))
    }
}

impl<C> From<&Term> for Expr<C>
where
    C: Zero + Clone + One + PartialEq + From<BigUint>,
{
    fn from(value: &Term) -> Self {
        let mut a = vec![];
        for (sym, deg) in &value.0 {
            a.push(Expr::Pow(
                Box::new(Expr::from(sym)),
                Box::new(Expr::Const(C::from(deg.clone()))),
            ));
        }
        Expr::Mul(a).flatten()
    }
}

impl<C> From<&Polynomial<C>> for Expr<C>
where
    C: Zero + Clone + One + PartialEq + From<BigUint>,
{
    fn from(value: &Polynomial<C>) -> Self {
        let mut a = Vec::new();
        for (term, coef) in &value.0 {
            a.push(Expr::Mul(vec![coef.clone(), Expr::from(term)]));
        }
        Expr::Add(a).flatten()
    }
}

static PRATT_PARSER: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    PrattParser::new()
        .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
        .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
        .op(Op::infix(Rule::pow, Assoc::Right))
});

impl<C> Expr<C>
where
    for<'a> C: FromStr + Zero + Clone + One + PartialEq + Neg<Output = C> + Div<Output = C>,
{
    pub fn from_str(
        input: &str,
        symbols: &mut HashMap<String, Arc<Symbol>>,
    ) -> Result<Self, Error<Rule>> {
        Ok(PRATT_PARSER
            .map_primary(|primary| match primary.as_rule() {
                Rule::symbol => {
                    if let Some(t) = symbols.get(primary.as_str()) {
                        Expr::from(t)
                    } else {
                        let sym = Arc::new(Symbol::new(primary.as_str()));
                        symbols.insert(primary.as_str().to_string(), Arc::clone(&sym));
                        Expr::from(&sym)
                    }
                }
                Rule::number => Expr::Const(match C::from_str(primary.as_str()) {
                    Ok(t) => t,
                    Err(_) => todo!(),
                }),
                _ => unreachable!(),
            })
            .map_infix(|lhs, op, rhs| match op.as_rule() {
                Rule::add => lhs + rhs,
                Rule::sub => lhs - rhs,
                Rule::mul => lhs * rhs,
                Rule::div => lhs / rhs,
                //TODO 或许有别的方式
                Rule::pow => Expr::Pow(Box::new(lhs.flatten()), Box::new(rhs.flatten())).flatten(),
                _ => unreachable!(),
            })
            .parse(Self::parse(Rule::expr, input)?.next().unwrap().into_inner()))
    }
}
