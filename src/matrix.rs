//! 行列とベクトルの計算

use std::ops::{Add, Deref, DerefMut};

#[allow(unused)]
pub struct Matrix(pub Vec<Vector>);

/// 成分が i64 のベクトル
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vector(pub Vec<i64>);

/// v[i] や v.iter() のように、中の Vec と同じく使えるようにする
impl Deref for Vector {
    type Target = [i64];

    fn deref(&self) -> &[i64] {
        &self.0
    }
}

impl DerefMut for Vector {
    fn deref_mut(&mut self) -> &mut [i64] {
        &mut self.0
    }
}

/// .collect() で Vector を作れるようにする
impl FromIterator<i64> for Vector {
    fn from_iter<I: IntoIterator<Item = i64>>(iter: I) -> Vector {
        Vector(iter.into_iter().collect())
    }
}

/// 成分ごとの足し算。c[i] = a[i] + b[i]
///
/// 参照どうしで足す（&a + &b）ので、a と b はその後も使える
impl Add for &Vector {
    type Output = Vector;

    fn add(self, other: &Vector) -> Vector {
        self.iter().zip(other.iter()).map(|(a, b)| a + b).collect()
    }
}

impl Vector {
    pub fn min_index(&self) -> usize {
        self.iter()
            .enumerate()
            .min_by_key(|&(_, &value)| value)
            .map(|(index, _)| index)
            .unwrap()
    }
}

// 遷移をつなぐ。c[i][j] = min_k (a[i][k] + b[k][j])
// pub fn compose(a: &Matrix, b: &Matrix) -> Matrix {
//     todo!()
// }

impl Deref for Matrix {
    type Target = [Vector];

    fn deref(&self) -> &[Vector] {
        &self.0
    }
}

/// .collect() で Matrix を作れるようにする（各行は Vector）
impl FromIterator<Vector> for Matrix {
    fn from_iter<I: IntoIterator<Item = Vector>>(iter: I) -> Matrix {
        Matrix(iter.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_is_elementwise() {
        let a = Vector(vec![1, 2, 3]);
        let b = Vector(vec![10, 20, 30]);
        assert_eq!(&a + &b, Vector(vec![11, 22, 33]));
    }

    #[test]
    fn min_index_works() {
        let a = Vector(vec![1, -1, 3]);
        assert_eq!(a.min_index(), 1);
    }
}
