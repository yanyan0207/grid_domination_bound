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

#[cfg(test)]
mod tests {
    use crate::{
        cost::stone_number,
        rules::{Edges, list_valid_rows},
    };

    use super::*;

    #[test_case::test_matrix(
    [2,3,4,5],[true,false],[true,false],[Problem::Dom2,Problem::Fair2]
)]
    fn test_matrix_invalid(width: usize, left: bool, right: bool, problem: Problem) {
        let edges = Edges {
            top: true,
            bottom: true,
            left,
            right,
        };

        let valid_rows = list_valid_rows(width, edges.row());

        let matrix = build_transition_cost_matrix(&valid_rows, problem, Cost::Loss, edges.row());

        for (prev_index, prev) in valid_rows.iter().enumerate() {
            for (next_index, next) in valid_rows.iter().enumerate() {
                let actual = matrix[prev_index][next_index] != INF;
                let mut expected = true;
                for i in 0..width {
                    if problem == Problem::Fair2 && prev[i] == Cell::Ok && next[i] == Cell::Stone {
                        expected = false;
                    }
                    if prev[i] == Cell::NeedOne && next[i] != Cell::Stone {
                        expected = false;
                    }
                    if next[i] != Cell::Stone {
                        let maybe = if (i == 0 && !left) || (i == width - 1 && !right) {
                            1
                        } else {
                            0
                        };
                        let mut around_stone = 0;
                        if i > 0 && next[i - 1] == Cell::Stone {
                            around_stone += 1;
                        }
                        if i < width - 1 && next[i + 1] == Cell::Stone {
                            around_stone += 1;
                        }
                        if prev[i] == Cell::Stone {
                            around_stone += 1;
                        }

                        if next[i] == Cell::Ok {
                            if around_stone + maybe < 2 {
                                expected = false;
                            }
                            if problem == Problem::Fair2 && around_stone > 2 {
                                expected = false;
                            }
                        }
                        if next[i] == Cell::NeedOne
                            && around_stone != 1
                            && around_stone + maybe != 1
                        {
                            expected = false;
                        }
                    }
                }
                assert_eq!(
                    actual, expected,
                    "prev: {:?}, next: {:?}, width: {}, left: {}, right: {}, problem: {:?}",
                    prev, next, width, left, right, problem
                );
            }
        }
    }

    #[test_case::test_matrix(
    [2,3,4,5],[2,3,4,5],[true],[true],[Problem::Dom2,Problem::Fair2]
)]
    fn test_transition_board_cost(
        width: usize,
        hight: usize,
        left: bool,
        right: bool,
        problem: Problem,
    ) {
        let edges = Edges {
            top: true,
            bottom: true,
            left,
            right,
        };

        let valid_rows = list_valid_rows(width, edges.row());
        let matrix: Matrix =
            build_transition_cost_matrix(&valid_rows, problem, Cost::Loss, edges.row());

        search(
            &mut vec![],
            &valid_rows,
            &matrix,
            Cost::Loss,
            hight,
            edges,
            problem,
        );

        fn check_path(
            path: &[usize],
            valid_rows: &[Vec<Cell>],
            matrix: &Matrix,
            cost: Cost,
            edges: Edges,
            problem: Problem,
        ) {
            let board = path
                .iter()
                .map(|index| &valid_rows[*index])
                .collect::<Vec<&Vec<Cell>>>();

            let mut stone_count = 0;
            let actual_cost = cost.first_row(board[0], &edges)
                + path.windows(2).map(|w| matrix[w[0]][w[1]]).sum::<i64>()
                + cost.last_row_addition(edges.row(), board[board.len() - 1]);

            let mut is_valid = true;
            for row in 0..board.len() {
                for col in 0..board[row].len() {
                    let mut arround_stone_count = 0;
                    if board[row][col] == Cell::Stone {
                        stone_count += 1;
                    } else {
                        if col > 0 && board[row][col - 1] == Cell::Stone {
                            arround_stone_count += 1;
                        }
                        if col < board[row].len() - 1 && board[row][col + 1] == Cell::Stone {
                            arround_stone_count += 1;
                        }
                        if row > 0 && board[row - 1][col] == Cell::Stone {
                            arround_stone_count += 1;
                        }
                        if row < board.len() - 1 && board[row + 1][col] == Cell::Stone {
                            arround_stone_count += 1;
                        }
                        if (problem == Problem::Fair2 && arround_stone_count != 2)
                            || (problem == Problem::Dom2 && arround_stone_count < 2)
                        {
                            is_valid = false;
                        }
                    }
                }
            }

            if is_valid {
                let actual_stone_count =
                    stone_number(cost, actual_cost, board.len(), board[0].len());
                assert_eq!(
                    actual_stone_count, stone_count,
                    "{}, {}, {}, {:?}",
                    actual_stone_count, actual_cost, stone_count, board
                );
            } else {
                assert!(actual_cost >= INF);
            }
        }

        fn search(
            path: &mut Vec<usize>,
            valid_rows: &Vec<Vec<Cell>>,
            matrix: &Matrix,
            cost: Cost,
            height: usize,
            edges: Edges,
            problem: Problem,
        ) {
            let valid_cnt = valid_rows.len();
            if path.is_empty() {
                for next_index in 0..valid_cnt {
                    if cost.first_row(&valid_rows[next_index], &edges) == INF {
                        continue;
                    }
                    path.push(next_index);
                    search(path, valid_rows, matrix, cost, height, edges, problem);
                    path.pop();
                }
            } else if path.len() == height {
                check_path(path, valid_rows, matrix, cost, edges, problem);
            } else {
                let prev_index = *path.last().unwrap();
                for next_index in 0..valid_cnt {
                    if matrix[prev_index][next_index] == INF {
                        continue;
                    }
                    path.push(next_index);
                    search(path, valid_rows, matrix, cost, height, edges, problem);
                    path.pop();
                }
            }
        }
    }
}
