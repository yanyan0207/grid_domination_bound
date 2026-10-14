//! 転送行列（行から行への遷移コスト）の作成

use crate::{
    cost::Cost,
    matrix::{Matrix, SparseMatrix},
    rules::{Cell, Problem, RowEdges, is_valid_transition},
};

pub const INF: i64 = 1_000_000_000;

/// 行 rows の間の遷移コスト行列を作る
///
/// 遷移可能な場合は cost(遷移元, 遷移先)、遷移不可能な場合は無限大とする
/// 行列は prev_row -> next_row の順（m[遷移元][遷移先]）。論文の T[S][S'] と同じ向きで、compose でそのままつなげる
pub fn build_transition_cost_matrix(
    rows: &[Vec<Cell>],
    problem: Problem,
    cost: Cost,
    row_edges: RowEdges,
) -> Matrix {
    rows.iter()
        .map(|prev_row| {
            rows.iter()
                .map(|next_row| {
                    if is_valid_transition(prev_row, next_row, problem, row_edges) {
                        cost.transition(prev_row, next_row, row_edges)
                    } else {
                        INF
                    }
                })
                .collect()
        })
        .collect()
}

/// 行 rows の間の遷移コスト行列を作る
///
/// 遷移可能な場合は cost(遷移元, 遷移先)、遷移不可能な場合は無限大とする
/// 計算の都合上、next_row -> prev_row の順で行列を作る
pub fn build_transition_costs(
    rows: &[Vec<Cell>],
    problem: Problem,
    cost: Cost,
    row_edges: RowEdges,
) -> SparseMatrix {
    // 遷移先の行ごとに有効な遷移を計算
    let mut transition_costs: SparseMatrix = vec![];
    for next_row in rows {
        // 遷移元の行から有効な遷移のみのコストを保存する
        let mut valid_transitions = vec![];
        for (j, prev_row) in rows.iter().enumerate() {
            if is_valid_transition(prev_row, next_row, problem, row_edges) {
                let value = cost.transition(prev_row, next_row, row_edges);
                valid_transitions.push((j, value));
            }
        }
        transition_costs.push(valid_transitions);
    }
    transition_costs
}

pub fn build_transition_pairs(
    rows: &[Vec<Cell>],
    problem: Problem,
    row_edges: RowEdges,
) -> Vec<Vec<usize>> {
    let mut ret = vec![];
    for prev_row in rows {
        let mut valid_next_rows = vec![];
        for (j, next_row) in rows.iter().enumerate() {
            if is_valid_transition(prev_row, next_row, problem, row_edges) {
                valid_next_rows.push(j);
            }
        }
        ret.push(valid_next_rows);
    }
    ret
}
