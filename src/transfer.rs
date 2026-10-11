//! 転送行列（行から行への遷移コスト）の作成

use crate::rules::{Cell, Problem, is_valid_transition};

pub const INF: i64 = 1_000_000_000;
type Matrix = Vec<Vec<i64>>;

/// 行 rows の間の遷移コスト行列を作る
///
/// 遷移可能な場合は cost(遷移先)、遷移不可能な場合は無限大とする
/// 計算の都合上、next_row -> prev_row の順で行列を作る
pub fn transition_cost_matrix(
    rows: &[Vec<Cell>],
    problem: Problem,
    top_is_edge: bool,
    bottom_is_edge: bool,
    next_row_costs: &[i64],
) -> Matrix {
    rows.iter()
        .zip(next_row_costs)
        .map(|(next_row, cost)| {
            rows.iter()
                .map(|prev_row| {
                    if is_valid_transition(prev_row, next_row, problem, top_is_edge, bottom_is_edge)
                    {
                        *cost
                    } else {
                        INF
                    }
                })
                .collect()
        })
        .collect()
}
