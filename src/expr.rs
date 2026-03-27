use crate::{forward_impl_binop, polynomial::Polynomial, symbol::Symbol, term::Term};
use num::{BigUint, One, Zero, pow::Pow};
use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
    sync::Arc,
};

#[derive(Debug, Clone)]
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
        match (self, other) {
            (Expr::Const(a), Expr::Const(b)) => a == b,
            (Expr::Symbol(a), Expr::Symbol(b)) => *a == *b,
            (Expr::Add(a), Expr::Add(b)) | (Expr::Mul(a), Expr::Mul(b)) if a.len() == b.len() => {
                let mut a = a.clone();
                for i in b {
                    if let Some(k) = a.iter().position(|x| *x == *i) {
                        a.remove(k);
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
                let mut has_neg = false;
                let mut s = vec![];
                for v in t.iter() {
                    let a = v.to_string();
                    match v {
                        //直接比较字符串, 就不需要对C进行别的约束
                        Expr::Const(_) if !has_neg && a == "-1" => has_neg = true,
                        Expr::Add(_) => s.push(format!("({a})")),
                        _ => s.push(a),
                    }
                }
                write!(f, "{}{}", if has_neg { "-" } else { "" }, s.join("*"))?;
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
    C: Zero + One + PartialEq + Clone + Pow<Expr<C>, Output = Expr<C>>,
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
    C: One + PartialEq + Zero + Clone + Pow<Expr<C>, Output = Expr<C>>,
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
    C: Zero + One + PartialEq + Clone + Pow<Expr<C>, Output = Expr<C>>,
{
    pub fn simplify(&self) -> Expr<C> {
        match self.flatten().evaluate().collect() {
            t if t.is_zero() => Expr::zero(),
            t if t.is_one() => Expr::one(),
            Expr::Pow(a, b) if b.is_one() => *a,
            t => t,
        }
    }

    pub fn flatten(&self) -> Expr<C> {
        match self {
            Expr::Add(t) => {
                let mut a = vec![];
                for i in t {
                    match i.flatten() {
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
                        Expr::Mul(t) => a.extend(t),
                        t => a.push(t),
                    }
                }
                if a.len() == 0 {
                    Expr::one()
                } else if a.len() == 1 {
                    a.remove(0)
                } else {
                    Expr::Mul(a)
                }
            }
            Expr::Pow(a, b) => Expr::Pow(Box::new(a.flatten()), Box::new(b.flatten())),
            _ => self.clone(),
        }
    }

    pub fn collect(&self) -> Expr<C> {
        match self {
            Expr::Add(t) => {
                let mut coefs = vec![];
                for i in t {
                    let (term, coef) = match i.collect() {
                        Expr::Mul(t) => {
                            let mut term = vec![];
                            let mut coef = C::one();
                            for i in t {
                                match i {
                                    Expr::Const(t) => coef = coef * t,
                                    _ => term.push(i),
                                }
                            }
                            (Expr::Mul(term).flatten(), coef)
                        }
                        t => (t, C::one()),
                    };
                    match coefs.iter().position(|x: &(Expr<C>, C)| x.0 == term) {
                        Some(i) => {
                            coefs.get_mut(i).unwrap().1 = coefs.get(i).unwrap().1.clone() + coef
                        }
                        None => coefs.push((term, coef)),
                    }
                }

                let mut a = vec![];
                for (term, coef) in coefs {
                    if term.is_zero() || coef.is_zero() {
                        continue;
                    }
                    if coef.is_one() {
                        a.push(term);
                    } else if term.is_one() {
                        a.push(Expr::Const(coef));
                    } else {
                        a.push(term * Expr::Const(coef));
                    }
                }
                Expr::Add(a).flatten()
            }
            Expr::Mul(t) => {
                let mut constant = C::one();
                let mut exps = vec![];
                for i in t {
                    let (base, exp) = match i.collect() {
                        Expr::Pow(a, b) => match *b {
                            Expr::Const(c) => (*a, c),
                            _ => (Expr::Pow(a, b), C::one()),
                        },
                        Expr::Const(t) => {
                            constant = constant * t;
                            continue;
                        }
                        t => (t, C::one()),
                    };
                    match exps.iter().position(|x: &(Expr<C>, C)| x.0 == base) {
                        Some(i) => {
                            exps.get_mut(i).unwrap().1 = exps.get(i).unwrap().1.clone() + exp
                        }
                        None => exps.push((base, exp)),
                    }
                }

                let mut a = if constant.is_one() {
                    vec![]
                } else {
                    vec![Expr::Const(constant)]
                };
                for (base, exp) in exps {
                    if base.is_one() || exp.is_zero() {
                        continue;
                    }
                    if exp.is_one() {
                        a.push(base);
                    } else {
                        a.push(Expr::Pow(Box::new(base), Box::new(Expr::Const(exp))));
                    }
                }
                Expr::Mul(a).flatten()
            }
            Expr::Pow(a, b) => Expr::Pow(Box::new(a.collect()), Box::new(b.collect())),
            _ => self.clone(),
        }
    }

    pub fn evaluate(&self) -> Expr<C> {
        match self {
            Expr::Add(t) => {
                let mut a = vec![];
                let mut constant = C::zero();
                for i in t {
                    match i.evaluate() {
                        Expr::Const(t) => constant = constant + t,
                        t => a.push(t),
                    }
                }
                if !constant.is_zero() {
                    a.push(Expr::Const(constant));
                }
                Expr::Add(a).flatten()
            }
            Expr::Mul(t) => {
                let mut a = vec![];
                let mut constant = C::one();
                for i in t {
                    match i.evaluate() {
                        Expr::Const(t) => constant = constant * t,
                        t => a.push(t),
                    }
                }
                if !constant.is_one() {
                    a.push(Expr::Const(constant));
                }
                Expr::Mul(a).flatten()
            }
            Expr::Pow(a, b) => match (a.evaluate(), b.evaluate()) {
                (Expr::Const(a), b) => a.pow(b),
                (a, b) => Expr::Pow(Box::new(a), Box::new(b)),
            }
            .flatten(),
            _ => self.clone(),
        }
    }

    pub fn expand(&self) -> Expr<C> {
        match self.simplify() {
            Expr::Add(t) => Expr::Add(t.iter().map(|x| x.expand()).collect()).simplify(),
            Expr::Mul(t) => {
                let mut a = vec![];

                for i in t {
                    match i.expand().simplify() {
                        Expr::Add(t) => {
                            if a.len() == 0 {
                                a.extend(t);
                            } else {
                                let mut b = vec![];
                                for i in &t {
                                    for j in &a {
                                        b.push(i * j);
                                    }
                                }
                                a = b;
                            }
                        }
                        t => {
                            if a.len() == 0 {
                                a.push(t);
                            } else {
                                let mut b = vec![];
                                for i in &a {
                                    b.push(i * &t);
                                }
                                a = b;
                            }
                        }
                    }
                }

                Expr::Add(a).simplify()
            }
            Expr::Pow(a, b) => match *b {
                Expr::Const(t) => {
                    //TODO 展开
                    Expr::Pow(Box::new(a.expand()), Box::new(Expr::Const(t))).simplify()
                }
                b => Expr::Pow(Box::new(a.expand()), Box::new(b.expand())).simplify(),
            },
            t => t,
        }
    }
}

impl<C> Add<&Expr<C>> for &Expr<C>
where
    C: Zero + Clone + One + PartialEq + Pow<Expr<C>, Output = Expr<C>>,
{
    type Output = Expr<C>;

    fn add(self, rhs: &Expr<C>) -> Self::Output {
        Expr::Add(vec![self.clone(), rhs.clone()]).simplify()
    }
}

forward_impl_binop!(impl<C> Add for Expr<C>, add, where C: Zero + Clone + One + PartialEq + Pow<Expr<C>, Output = Expr<C>>);

impl<C> Sub<&Expr<C>> for &Expr<C>
where
    C: Zero + Clone + One + PartialEq + Neg<Output = C> + Pow<Expr<C>, Output = Expr<C>>,
{
    type Output = Expr<C>;

    fn sub(self, rhs: &Expr<C>) -> Self::Output {
        Expr::Add(vec![self.clone(), -rhs.clone()]).simplify()
    }
}

forward_impl_binop!(
    impl<C> Sub for Expr<C>,
    sub,
    where
        C: Zero + Clone + One + PartialEq + Neg<Output = C> + Pow<Expr<C>, Output = Expr<C>>
);

impl<C> Mul<&Expr<C>> for &Expr<C>
where
    C: Zero + Clone + One + PartialEq + Pow<Expr<C>, Output = Expr<C>>,
{
    type Output = Expr<C>;

    fn mul(self, rhs: &Expr<C>) -> Self::Output {
        Expr::Mul(vec![self.clone(), rhs.clone()]).simplify()
    }
}

forward_impl_binop!(impl<C> Mul for Expr<C>,mul,where C: Zero + Clone + One + PartialEq + Pow<Expr<C>, Output = Expr<C>>);

impl<C> Div<&Expr<C>> for &Expr<C>
where
    C: Div<Output = C>
        + Zero
        + Clone
        + One
        + PartialEq
        + Neg<Output = C>
        + Pow<Expr<C>, Output = Expr<C>>,
{
    type Output = Expr<C>;

    fn div(self, rhs: &Expr<C>) -> Self::Output {
        Expr::Mul(vec![
            self.clone(),
            Expr::Pow(Box::new(rhs.clone()), Box::new(Expr::Const(-C::one()))),
        ])
        .simplify()
    }
}

forward_impl_binop!(
    impl<C> Div for Expr<C>,
    div,
    where
        C: Div<Output = C> + Zero + Clone + One + PartialEq + Neg<Output = C> + Pow<Expr<C>, Output = Expr<C>>
);

impl<C> Pow<&Expr<C>> for &Expr<C>
where
    C: Zero + Clone + One + PartialEq + Pow<Expr<C>, Output = Expr<C>>,
{
    type Output = Expr<C>;

    fn pow(self, rhs: &Expr<C>) -> Self::Output {
        Expr::Pow(Box::new(self.clone()), Box::new(rhs.clone())).simplify()
    }
}

forward_impl_binop!(
    impl<C> Pow for Expr<C>,
    pow,
    where
        C: Zero + Clone + One + PartialEq + Pow<Expr<C>, Output = Expr<C>>
);

impl<C> Neg for &Expr<C>
where
    C: Neg<Output = C> + Clone + Zero + One + PartialEq + Pow<Expr<C>, Output = Expr<C>>,
{
    type Output = Expr<C>;

    fn neg(self) -> Self::Output {
        Expr::Mul(vec![Expr::Const(-C::one()), self.clone()]).simplify()
    }
}

impl<C> Neg for Expr<C>
where
    C: Neg<Output = C> + Clone + Zero + One + PartialEq + Pow<Expr<C>, Output = Expr<C>>,
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
    C: Zero + Clone + One + PartialEq + From<BigUint> + Pow<Expr<C>, Output = Expr<C>>,
{
    fn from(value: &Term) -> Self {
        let mut a = Expr::one();
        for (sym, deg) in &value.0 {
            a = a * Expr::Pow(
                Box::new(Expr::from(sym)),
                Box::new(Expr::Const(C::from(deg.clone()))),
            );
        }
        a
    }
}

impl<C> From<&Polynomial<C>> for Expr<C>
where
    C: Zero + Clone + One + PartialEq + From<BigUint> + Pow<Expr<C>, Output = Expr<C>>,
{
    fn from(value: &Polynomial<C>) -> Self {
        let mut a = Expr::zero();
        for (term, coef) in &value.0 {
            a = a + coef * Expr::from(term);
        }
        a
    }
}
