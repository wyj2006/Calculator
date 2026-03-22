use crate::{expr::Expr, forward_impl_binop, symbol::Symbol, term::Term};
use num::{BigUint, One, Zero, pow::Pow};
use std::{
    collections::BTreeMap,
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
    sync::Arc,
};

#[derive(Debug, Clone)]
pub struct Polynomial<C>(pub BTreeMap<Term, Expr<C>>);

impl<C> Zero for Polynomial<C>
where
    C: Zero + Clone + One + PartialEq,
{
    fn is_zero(&self) -> bool {
        self.0.values().all(|x| x.is_zero())
    }

    fn zero() -> Self {
        Polynomial(BTreeMap::from([(Term(BTreeMap::new()), Expr::zero())]))
    }
}

impl<C> One for Polynomial<C>
where
    C: Zero + Clone + One + PartialEq,
{
    fn is_one(&self) -> bool {
        for (term, coef) in &self.0 {
            if term.0.len() != 0 && !coef.is_zero() {
                return false;
            }
            if term.0.len() == 0 && !coef.is_one() {
                return false;
            }
        }
        true
    }

    fn one() -> Self {
        Polynomial(BTreeMap::from([(Term(BTreeMap::new()), Expr::one())]))
    }
}

impl<C> Polynomial<C>
where
    C: Zero + Clone + One + PartialEq,
{
    pub fn degree(&self, sym: &Arc<Symbol>) -> BigUint {
        let mut deg = BigUint::zero();
        for (term, coef) in &self.0 {
            if coef.is_zero() {
                continue;
            }
            deg = deg.max(term.degree(sym));
        }
        deg
    }

    pub fn total_degree(&self) -> BigUint {
        let mut tdeg = BigUint::zero();
        for (term, coef) in &self.0 {
            if coef.is_zero() {
                continue;
            }
            tdeg = tdeg.max(term.total_degree());
        }
        tdeg
    }
}

impl<C> Polynomial<C>
where
    C: Zero + Clone + One + PartialEq + From<BigUint>,
{
    pub fn leading_coef(&self, sym: &Arc<Symbol>) -> Expr<C> {
        let mut a = Expr::zero();
        let deg = self.degree(sym);
        for (term, coef) in &self.0 {
            if term.degree(sym) != deg {
                continue;
            }
            let mut term = term.clone();
            term.0.remove(sym);
            a = a + coef * Expr::from(&term);
        }
        a
    }
}

impl<C> Polynomial<C> {
    pub fn new_with_coef(coef: Expr<C>) -> Polynomial<C> {
        Polynomial(BTreeMap::from([(Term(BTreeMap::new()), coef)]))
    }
}

impl<C> Polynomial<C>
where
    C: Zero + Clone + One + PartialEq + From<BigUint>,
{
    ///看成关于symbol的一元多项式
    pub fn regard_as(&self, symbol: &Arc<Symbol>) -> Polynomial<C> {
        let mut t = BTreeMap::new();

        for (term, coef) in &self.0 {
            let mut new_term = Term::empty();
            let mut new_coef = coef.clone();

            for (sym, deg) in &term.0 {
                if *sym == *symbol {
                    new_term.0.insert(Arc::clone(sym), deg.clone());
                } else {
                    new_coef = new_coef
                        * Expr::Pow(
                            Box::new(Expr::from(sym)),
                            Box::new(Expr::Const(C::from(deg.clone()))),
                        )
                }
            }

            t.insert(new_term, new_coef);
        }

        Polynomial(t)
    }

    ///求导
    pub fn diff(&self, symbol: &Arc<Symbol>) -> Polynomial<C> {
        let mut t = BTreeMap::new();

        for (term, coef) in &self.0 {
            if term.0.len() == 0 {
                //常数
                continue;
            }
            let mut new_term = Term::empty();
            let mut new_coef = coef.clone();

            for (sym, deg) in &term.0 {
                if *sym == *symbol {
                    new_coef = new_coef * Expr::Const(C::from(deg.clone()));
                    new_term.0.insert(Arc::clone(sym), deg - BigUint::one());
                } else {
                    //不看成关于symbol的一元多项式
                    new_term.0.insert(Arc::clone(sym), deg.clone());
                }
            }

            t.insert(new_term, new_coef);
        }

        Polynomial(t)
    }
}

impl<C> Polynomial<C>
where
    C: Zero
        + Clone
        + One
        + PartialEq
        + From<BigUint>
        + Sub<Output = C>
        + Neg<Output = C>
        + Div<Output = C>,
{
    pub fn divmod(&self, rhs: &Polynomial<C>, sym: &Arc<Symbol>) -> (Polynomial<C>, Polynomial<C>) {
        let mut q = Self::zero();
        let mut r = self.regard_as(sym);

        let g = rhs.regard_as(sym);
        let l = g.degree(sym);

        while r.degree(sym) >= l {
            let a = Polynomial::new_with_coef(r.leading_coef(sym) / g.leading_coef(sym))
                * Self::from(&Term::from(sym).pow(r.degree(sym) - &l));
            r = r - &a * &g;
            q = q + a;
        }

        (q, r)
    }

    pub fn pseudo_divmod(
        &self,
        rhs: &Polynomial<C>,
        sym: &Arc<Symbol>,
    ) -> (Polynomial<C>, Polynomial<C>, BigUint) {
        let mut q = Self::zero();
        let mut r = self.regard_as(&sym);
        let mut s = BigUint::zero();

        let g = rhs.regard_as(&sym);
        let l = g.degree(sym);
        let a = Polynomial::new_with_coef(g.leading_coef(sym));

        while r.degree(sym) >= l {
            let b = Polynomial::new_with_coef(r.leading_coef(sym))
                * Self::from(&Term::from(sym).pow(r.degree(sym) - &l));
            r = r * &a - &b * &g;
            q = q * &a + b;
            s = s + BigUint::one();
        }

        (q, r, s)
    }
}

impl<C> Display for Polynomial<C>
where
    C: Display + Zero + Clone + One + PartialEq + From<BigUint>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Expr::from(self))
    }
}

impl<C> TryFrom<&Expr<C>> for Polynomial<C>
where
    C: Zero + One + PartialEq + Clone,
    BigUint: TryFrom<C>,
{
    type Error = ();

    fn try_from(value: &Expr<C>) -> Result<Self, Self::Error> {
        match value.flatten() {
            t @ Expr::Const(_) => Ok(Polynomial::new_with_coef(t)),
            Expr::Symbol(t) => Ok(Polynomial(BTreeMap::from([(
                Term(BTreeMap::from([(t, BigUint::one())])),
                Expr::Const(C::one()),
            )]))),
            Expr::Add(t) => {
                let mut a = Polynomial::zero();
                for i in t {
                    a = a + Polynomial::try_from(&i)?;
                }
                Ok(a)
            }
            Expr::Mul(t) => {
                let mut a = Polynomial::one();
                for i in t {
                    a = a * Polynomial::try_from(&i)?;
                }
                Ok(a)
            }
            Expr::Pow(a, b) => match (*a, *b) {
                (Expr::Symbol(a), Expr::Const(b)) => Ok(Polynomial(BTreeMap::from([(
                    Term(BTreeMap::from([(a, BigUint::try_from(b).map_err(|_| ())?)])),
                    Expr::Const(C::one()),
                )]))),
                _ => Err(()),
            },
        }
    }
}

impl<C> Add<&Polynomial<C>> for &Polynomial<C>
where
    C: Zero + Clone + One + PartialEq,
{
    type Output = Polynomial<C>;

    fn add(self, rhs: &Polynomial<C>) -> Self::Output {
        let mut t = BTreeMap::new();

        for (term, b) in self.0.iter().chain(&rhs.0) {
            match t.get(term) {
                Some(a) => t.insert(term.clone(), a + b),
                None => t.insert(term.clone(), b.flatten()),
            };
        }

        Polynomial(t)
    }
}

forward_impl_binop!(
    impl<C> Add for Polynomial<C>,
    add,
    where C: Zero + Clone + One + PartialEq
);

impl<C> Sub<&Polynomial<C>> for &Polynomial<C>
where
    C: Zero + Clone + One + PartialEq + Sub<Output = C> + Neg<Output = C>,
{
    type Output = Polynomial<C>;

    fn sub(self, rhs: &Polynomial<C>) -> Self::Output {
        let mut t = self.0.clone();

        for (term, b) in &rhs.0 {
            match t.get(term) {
                Some(a) => t.insert(term.clone(), a - b),
                None => t.insert(term.clone(), -b.flatten()),
            };
        }

        Polynomial(t)
    }
}

forward_impl_binop!(
    impl<C> Sub for Polynomial<C>,
    sub,
    where C: Zero + Clone + One + PartialEq + Sub<Output = C> + Neg<Output = C>,
);

impl<C> Mul<&Polynomial<C>> for &Polynomial<C>
where
    C: Zero + Clone + One + PartialEq,
{
    type Output = Polynomial<C>;

    fn mul(self, rhs: &Polynomial<C>) -> Self::Output {
        let mut t = BTreeMap::new();

        for (x1, x2) in &self.0 {
            for (y1, y2) in &rhs.0 {
                let term = x1 * y1;
                let b = x2 * y2;
                match t.get(&term) {
                    Some(a) => t.insert(term, a + b),
                    None => t.insert(term, b),
                };
            }
        }

        Polynomial(t)
    }
}

forward_impl_binop!(
    impl<C> Mul for Polynomial<C>,
    mul,
    where C: Zero + Clone + One + PartialEq
);

impl<C> From<&Term> for Polynomial<C>
where
    C: Zero + Clone + One + PartialEq,
{
    fn from(value: &Term) -> Self {
        Polynomial(BTreeMap::from([(value.clone(), Expr::one())]))
    }
}
