use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Sub},
    str::FromStr,
};

use bigdecimal::BigDecimal;
use num::{
    BigInt, BigRational, BigUint, One, Zero, bigint::TryFromBigIntError, rational::ParseRatioError,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rational(BigRational);

impl TryFrom<Rational> for BigUint {
    type Error = TryFromBigIntError<BigInt>;

    fn try_from(value: Rational) -> Result<Self, Self::Error> {
        BigUint::try_from(
            (BigDecimal::from(value.0.numer().clone()) / BigDecimal::from(value.0.denom().clone()))
                .into_bigint_and_exponent()
                .0,
        )
    }
}

// impl From<BigUint> for Rational {
//     fn from(value: BigUint) -> Self {
//         Rational(BigRational::from(BigInt::from(value)))
//     }
// }

impl<T> From<T> for Rational
where
    BigInt: From<T>,
{
    fn from(value: T) -> Self {
        Rational(BigRational::from(BigInt::from(value)))
    }
}

impl FromStr for Rational {
    type Err = ParseRatioError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Rational(BigRational::from_str(s)?))
    }
}

impl Display for Rational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Zero for Rational {
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    fn zero() -> Self {
        Rational(BigRational::zero())
    }
}

impl One for Rational {
    fn is_one(&self) -> bool
    where
        Self: PartialEq,
    {
        self.0.is_one()
    }

    fn one() -> Self {
        Rational(BigRational::one())
    }
}

impl Add for Rational {
    type Output = Rational;

    fn add(self, rhs: Self) -> Self::Output {
        Rational(self.0 + rhs.0)
    }
}

impl Sub for Rational {
    type Output = Rational;

    fn sub(self, rhs: Self) -> Self::Output {
        Rational(self.0 - rhs.0)
    }
}

impl Mul for Rational {
    type Output = Rational;

    fn mul(self, rhs: Self) -> Self::Output {
        Rational(self.0 * rhs.0)
    }
}

impl Div for Rational {
    type Output = Rational;

    fn div(self, rhs: Self) -> Self::Output {
        Rational(self.0 / rhs.0)
    }
}

impl Neg for Rational {
    type Output = Rational;

    fn neg(self) -> Self::Output {
        Rational(-self.0)
    }
}
