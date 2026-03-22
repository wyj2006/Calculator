use std::{
    fmt::Display,
    ops::{Add, Index, IndexMut, Mul, Sub},
};

use num::{One, Zero};

#[derive(Debug, Clone)]
pub struct Matrix<D>(pub Vec<Vec<D>>);

impl<D> Index<(usize, usize)> for Matrix<D> {
    type Output = D;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.0[index.0][index.1]
    }
}

impl<D> IndexMut<(usize, usize)> for Matrix<D> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.0[index.0][index.1]
    }
}

impl<D> Matrix<D> {
    pub fn column(&self) -> usize {
        self.0.iter().map(|x| x.len()).min().unwrap_or(0)
    }

    pub fn row(&self) -> usize {
        self.0.len()
    }
}

impl<D> Matrix<D>
where
    D: Zero + Clone,
{
    pub fn new_zeros(row: usize, col: usize) -> Matrix<D> {
        Matrix(vec![vec![D::zero(); col]; row])
    }
}

impl<D> Matrix<D>
where
    D: Zero + Clone + Add<Output = D> + Sub<Output = D> + Mul<Output = D> + One,
{
    pub fn det(&self) -> D {
        if self.row() != self.column() {
            panic!("not a square matrix");
        }

        let mut res = D::zero();
        let n = self.row();

        match n {
            0 => D::one(),
            1 => self.0[0][0].clone(),
            2 => {
                self.0[0][0].clone() * self.0[1][1].clone()
                    - self.0[0][1].clone() * self.0[1][0].clone()
            }
            _ => {
                //TODO 更快的算法
                for i in 0..n {
                    let mut sub_matrix = self.clone();
                    sub_matrix.0.remove(0);

                    for x in 0..n - 1 {
                        sub_matrix.0[x].remove(i);
                    }

                    let c = sub_matrix.det() * self.0[0][i].clone();
                    if i % 2 == 0 {
                        res = res + c;
                    } else {
                        res = res - c;
                    }
                }

                res
            }
        }
    }
}

impl<D> Display for Matrix<D>
where
    D: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (i, x) in self.0.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "[")?;
            for (j, y) in x.iter().enumerate() {
                if j > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{y}")?;
            }
            write!(f, "]")?;
        }
        write!(f, "]")?;
        Ok(())
    }
}
