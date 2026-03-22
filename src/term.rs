use crate::{expr::Expr, forward_impl_binop, symbol::Symbol};
use num::{BigUint, One, Zero, pow::Pow};
use std::{cmp::Ordering, collections::BTreeMap, fmt::Display, hash::Hash, ops::Mul, sync::Arc};

#[derive(Debug, Clone, Eq, Ord)]
pub struct Term(pub BTreeMap<Arc<Symbol>, BigUint>);

impl Term {
    pub fn empty() -> Term {
        Term(BTreeMap::new())
    }

    pub fn degree(&self, sym: &Arc<Symbol>) -> BigUint {
        match self.0.get(sym) {
            Some(t) => t.clone(),
            None => BigUint::zero(),
        }
    }

    pub fn total_degree(&self) -> BigUint {
        self.0.values().sum()
    }
}

impl PartialEq for Term {
    fn eq(&self, other: &Self) -> bool {
        for (sym, deg) in &other.0 {
            if self.degree(sym) != *deg {
                return false;
            }
        }
        true
    }
}

impl Hash for Term {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (sym, deg) in &self.0 {
            if deg.is_zero() {
                continue;
            }
            (sym, deg).hash(state);
        }
    }
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for (sym, deg) in &other.0 {
            match self.degree(&sym).partial_cmp(deg) {
                Some(Ordering::Greater) => return Some(Ordering::Greater),
                Some(Ordering::Less) => return Some(Ordering::Less),
                _ => {}
            }
        }
        Some(Ordering::Equal)
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Expr::<BigUint>::from(self))
    }
}

impl Mul<&Term> for &Term {
    type Output = Term;

    fn mul(self, rhs: &Term) -> Self::Output {
        let mut t: BTreeMap<Arc<Symbol>, BigUint> = BTreeMap::new();

        for (sym, b) in self.0.iter().chain(&rhs.0) {
            match t.get(sym) {
                Some(a) => t.insert(sym.clone(), a.clone() + b),
                None => t.insert(sym.clone(), b.clone()),
            };
        }

        Term(t)
    }
}

forward_impl_binop!(impl Mul for Term,mul);

impl Pow<&BigUint> for &Term {
    type Output = Term;

    fn pow(self, rhs: &BigUint) -> Self::Output {
        let mut t = BTreeMap::new();

        for (sym, b) in &self.0 {
            t.insert(sym.clone(), rhs * b);
        }

        Term(t)
    }
}

forward_impl_binop!(impl Pow<BigUint> for Term,pow);

impl From<&Arc<Symbol>> for Term {
    fn from(value: &Arc<Symbol>) -> Self {
        Term(BTreeMap::from([(value.clone(), BigUint::one())]))
    }
}
