//! 行列とベクトルの計算

use std::{
    fmt,
    ops::{Add, Deref, DerefMut},
};

use crate::transfer::INF;

/// 成分が i64 の行列。各行は Vector
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix(pub Vec<Vector>);
pub type SparseMatrix = Vec<Vec<(usize, i64)>>;

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
#[allow(dead_code)]
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

/// compose と同じ積を計算し、どこを経由したかも返す
///
/// 戻り値の via[i][j] は、c[i][j] が最小になったときの経由先 k（i → k → j）。
/// c[i][j] が INF（i から j へ行けない）なら None。最小が同じ k が複数あるときは、番号の小さい方
#[allow(dead_code)]
pub fn compose_with_via(a: &Matrix, b: &Matrix) -> (Matrix, Vec<Vec<Option<usize>>>) {
    let cols = b.first().map_or(0, |row| row.len());
    let mut c = Vec::with_capacity(a.len());
    let mut via = Vec::with_capacity(a.len());
    for a_row in a.iter() {
        let mut c_row = vec![INF; cols];
        let mut via_row = vec![None; cols];
        for (k, &a_ik) in a_row.iter().enumerate() {
            // 来られない経由先は飛ばす
            if a_ik >= INF {
                continue;
            }
            for ((c_ij, via_ij), &b_kj) in c_row.iter_mut().zip(via_row.iter_mut()).zip(b[k].iter())
            {
                let value = a_ik + b_kj;
                if value < *c_ij {
                    *c_ij = value;
                    *via_ij = Some(k);
                }
            }
        }
        c.push(Vector(c_row));
        via.push(via_row);
    }
    (Matrix(c), via)
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

impl Matrix {
    /// 転置する。t[j][i] = m[i][j]
    ///
    /// m[遷移元][遷移先] の行列なら、m[遷移先][遷移元] の向きになる
    #[allow(dead_code)]
    pub fn transpose(&self) -> Matrix {
        let cols = self.first().map_or(0, |row| row.len());
        (0..cols)
            .map(|j| self.iter().map(|row| row[j]).collect())
            .collect()
    }

    /// INF でない成分だけを持つ形にする。行 i は (列 j, m[i][j]) の一覧
    ///
    /// 行と列の向きはそのまま。m[遷移元][遷移先] なら、行ごとに遷移先の一覧になる
    #[allow(dead_code)]
    pub fn to_sparse(&self) -> SparseMatrix {
        self.iter()
            .map(|row| {
                row.iter()
                    .enumerate()
                    .filter(|&(_, &value)| value < INF)
                    .map(|(j, &value)| (j, value))
                    .collect()
            })
            .collect()
    }
}

/// 成分を表示用の文字列にする。INF 以上は「.」
fn cell_text(value: i64) -> String {
    if value >= INF {
        ".".to_string()
    } else {
        value.to_string()
    }
}

/// 1 行に並べて表示する。INF は「.」
impl fmt::Display for Vector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let texts: Vec<String> = self.iter().map(|&value| cell_text(value)).collect();
        write!(f, "{}", texts.join(" "))
    }
}

/// 1 行ずつ、桁をそろえて表示する。INF は「.」
///
/// 1 行目に列の番号、各行の先頭に行の番号を付ける
impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let texts: Vec<Vec<String>> = self
            .iter()
            .map(|row| row.iter().map(|&value| cell_text(value)).collect())
            .collect();
        let cols = texts.first().map_or(0, |row| row.len());
        // 成分の幅は、成分と列の番号のうち一番長いもの
        let width = texts
            .iter()
            .flatten()
            .map(|text| text.len())
            .chain((0..cols).map(|j| j.to_string().len()))
            .max()
            .unwrap_or(1);
        // 行の番号の幅
        let label_width = texts.len().saturating_sub(1).to_string().len();

        // 列の番号
        let header: Vec<String> = (0..cols).map(|j| format!("{j:>width$}")).collect();
        write!(f, "{:label_width$}  {}", "", header.join(" "))?;
        for (i, row) in texts.iter().enumerate() {
            let line: Vec<String> = row.iter().map(|text| format!("{text:>width$}")).collect();
            write!(f, "\n{i:>label_width$}: {}", line.join(" "))?;
        }
        Ok(())
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
    fn compose_with_via_records_intermediate() {
        let t = matrix(vec![
            vec![INF, 2, INF, INF],
            vec![0, INF, INF, 1],
            vec![INF, INF, INF, INF],
            vec![INF, 3, INF, INF],
        ]);
        let (c, via) = compose_with_via(&t, &t);
        assert_eq!(c, compose(&t, &t));
        assert_eq!(
            via,
            vec![
                vec![Some(1), None, None, Some(1)],
                // 1→0→1（0+2）の方が 1→3→1（1+3）より小さいので、経由先は 0
                vec![None, Some(0), None, None],
                vec![None, None, None, None],
                vec![Some(1), None, None, Some(1)],
            ]
        );
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
    fn display_aligns_and_hides_infinity() {
        let m = matrix(vec![vec![0, 12, INF], vec![INF, 3, 100]]);
        assert_eq!(
            m.to_string(),
            "     0   1   2\n0:   0  12   .\n1:   .   3 100"
        );
        assert_eq!(Vector(vec![1, INF, 23]).to_string(), "1 . 23");
    }

    #[test]
    fn transpose_swaps_rows_and_columns() {
        let m = matrix(vec![vec![1, 2, 3], vec![4, 5, INF]]);
        let expected = matrix(vec![vec![1, 4], vec![2, 5], vec![3, INF]]);
        assert_eq!(m.transpose(), expected);
        assert_eq!(m.transpose().transpose(), m);
    }

    #[test]
    fn to_sparse_keeps_finite_entries() {
        let m = matrix(vec![vec![INF, 2, 0], vec![INF, INF, INF]]);
        let expected: SparseMatrix = vec![vec![(1, 2), (2, 0)], vec![]];
        assert_eq!(m.to_sparse(), expected);
    }

    #[test]
    fn compose_infinity_does_not_grow() {
        let t = matrix(vec![vec![INF; 2]; 2]);
        assert_eq!(compose(&t, &t), t);
    }
}
