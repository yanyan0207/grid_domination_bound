//! 転送行列（行から行への遷移コスト）の作成

use crate::{
    matrix::Matrix,
    rules::{Cell, Problem, is_valid_transition},
};

pub const INF: i64 = 1_000_000_000;

/// 行 rows の間の遷移コスト行列を作る
///
/// 遷移可能な場合は cost(遷移元, 遷移先)、遷移不可能な場合は無限大とする
/// 計算の都合上、next_row -> prev_row の順で行列を作る
pub fn build_transition_cost_matrix(
    rows: &[Vec<Cell>],
    problem: Problem,
    left_is_edge: bool,
    right_is_edge: bool,
    cost: impl Fn(&[Cell], &[Cell]) -> i64,
) -> Matrix {
    rows.iter()
        .map(|next_row| {
            rows.iter()
                .map(|prev_row| {
                    if is_valid_transition(prev_row, next_row, problem, left_is_edge, right_is_edge)
                    {
                        cost(prev_row, next_row)
                    } else {
                        INF
                    }
                })
                .collect()
        })
        .collect()
}

pub fn build_transition_pairs(
    rows: &[Vec<Cell>],
    problem: Problem,
    left_is_edge: bool,
    right_is_edge: bool,
) -> Vec<Vec<usize>> {
    let mut ret = vec![];
    for prev_row in rows {
        let mut valid_next_rows = vec![];
        for (j, next_row) in rows.iter().enumerate() {
            if is_valid_transition(prev_row, next_row, problem, left_is_edge, right_is_edge) {
                valid_next_rows.push(j);
            }
        }
        ret.push(valid_next_rows);
    }
    ret
}
