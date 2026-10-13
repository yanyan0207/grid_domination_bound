//! 行列とベクトルの計算

use std::ops::{Add, Deref, DerefMut};

use crate::transfer::INF;

/// 成分が i64 の行列。各行は Vector
#[allow(unused)]
#[derive(Clone, Debug, PartialEq, Eq)]
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

/// 遷移をつなぐ（(min,+) の行列の積）。c[i][j] = min_k (a[i][k] + b[k][j])
///
/// a[i][k] を i から k、b[k][j] を k から j へのコストとすると、c[i][j] は k を経由して i から j へ行く最小コスト。
/// 行列は m[遷移元][遷移先] の向きで渡す。INF どうしを足しても INF より大きくならないようにする
pub fn compose(a: &Matrix, b: &Matrix) -> Matrix {
    let cols = b.first().map_or(0, |row| row.len());
    a.iter()
        .map(|a_row| {
            let mut c_row = vec![INF; cols];
            for (k, &a_ik) in a_row.iter().enumerate() {
                // 来られない経由先は飛ばす
                if a_ik >= INF {
                    continue;
                }
                for (c_ij, &b_kj) in c_row.iter_mut().zip(b[k].iter()) {
                    *c_ij = (*c_ij).min(a_ik + b_kj);
                }
            }
            c_row.into_iter().map(|value| value.min(INF)).collect()
        })
        .collect()
}

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

    fn matrix(rows: Vec<Vec<i64>>) -> Matrix {
        rows.into_iter().map(Vector).collect()
    }

    #[test]
    fn compose_two_steps() {
        let t = matrix(vec![
            vec![INF, 2, INF, INF],
            vec![0, INF, INF, 1],
            vec![INF, INF, INF, INF],
            vec![INF, 3, INF, INF],
        ]);
        let expected = matrix(vec![
            vec![2, INF, INF, 3],
            // 1→0→1 は 0+2、1→3→1 は 1+3。小さい方
            vec![INF, 2, INF, INF],
            vec![INF, INF, INF, INF],
            vec![3, INF, INF, 4],
        ]);
        assert_eq!(compose(&t, &t), expected);
    }

    #[test]
    fn compose_order_matters() {
        // a のあとに b。0→1 が 1、1→0 が 10
        let a = matrix(vec![vec![INF, 1], vec![INF, INF]]);
        let b = matrix(vec![vec![INF, INF], vec![10, INF]]);
        assert_eq!(compose(&a, &b), matrix(vec![vec![11, INF], vec![INF, INF]]));
        assert_eq!(compose(&b, &a), matrix(vec![vec![INF, INF], vec![INF, 11]]));
    }

    #[test]
    fn compose_infinity_does_not_grow() {
        let t = matrix(vec![vec![INF; 2]; 2]);
        assert_eq!(compose(&t, &t), t);
    }
}
