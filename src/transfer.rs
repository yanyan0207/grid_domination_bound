//! 転送行列（行から行への遷移コスト）の作成

use std::collections::HashMap;

use crate::{
    cost::Cost,
    matrix::{Matrix, SparseMatrix, Vector},
    rules::{Cell, Graph, Problem, RowEdges, list_next_rows},
};

pub const INF: i64 = 1_000_000_000;

/// 行 rows の間の遷移コスト行列を作る
///
/// 遷移可能な場合は cost(遷移元, 遷移先)、遷移不可能な場合は無限大とする
/// 行列は prev_row -> next_row の順（m[遷移元][遷移先]）。論文の T[S][S'] と同じ向きで、compose でそのままつなげる
pub fn build_transition_cost_matrix(
    rows: &[Vec<Cell>],
    graph: Graph,
    problem: Problem,
    cost: Cost,
    row_edges: RowEdges,
) -> Matrix {
    let mut matrix = vec![Vector(vec![INF; rows.len()]); rows.len()];
    for_each_transition(rows, graph, problem, cost, row_edges, |prev_index, next_index, value| {
        matrix[prev_index][next_index] = value;
    });
    Matrix(matrix)
}

/// 行 rows の間の遷移コスト行列を作る
///
/// 遷移可能な場合は cost(遷移元, 遷移先)、遷移不可能な場合は無限大とする
/// 計算の都合上、next_row -> prev_row の順で行列を作る
pub fn build_transition_costs(
    rows: &[Vec<Cell>],
    graph: Graph,
    problem: Problem,
    cost: Cost,
    row_edges: RowEdges,
) -> SparseMatrix {
    // 遷移先の行ごとに、遷移元の行とコストの一覧
    let mut transition_costs: SparseMatrix = vec![vec![]; rows.len()];
    for_each_transition(rows, graph, problem, cost, row_edges, |prev_index, next_index, value| {
        transition_costs[next_index].push((prev_index, value));
    });
    transition_costs
}

/// 行 rows の間の有効な遷移を全部見つけて、(遷移元の番号, 遷移先の番号, コスト) ごとに on_transition を呼ぶ
///
/// 遷移元の行ごとに、遷移できる次の行を作って（list_next_rows）rows の中の番号を引く。
/// 有効な行どうしの全部の組を調べるより、遷移が少ないときにずっと速い。
/// 遷移元の番号の小さい順に呼ぶので、同じ遷移先の一覧も遷移元の番号の順になる
fn for_each_transition(
    rows: &[Vec<Cell>],
    graph: Graph,
    problem: Problem,
    cost: Cost,
    row_edges: RowEdges,
    mut on_transition: impl FnMut(usize, usize, i64),
) {
    let index_of_row: HashMap<&[Cell], usize> = rows
        .iter()
        .enumerate()
        .map(|(index, row)| (row.as_slice(), index))
        .collect();
    for (prev_index, prev_row) in rows.iter().enumerate() {
        for next_row in list_next_rows(prev_row, graph, problem, row_edges) {
            // rows にない行（rows が有効な行の一部だけのとき）への遷移は使わない
            if let Some(&next_index) = index_of_row.get(next_row.as_slice()) {
                let value = cost.transition(graph, prev_row, &next_row, row_edges);
                on_transition(prev_index, next_index, value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        cost::stone_number,
        rules::{
            ColumnPlace, Edges, SAME_ROW_COLUMN_OFFSETS, column_place, is_valid_transition,
            list_valid_rows,
        },
    };

    use super::*;

    /// 有効な行どうしの全部の組を調べて遷移コスト行列を作る（list_next_rows を使う本体と比べるため）
    fn build_transition_costs_by_all_pairs(
        rows: &[Vec<Cell>],
        graph: Graph,
        problem: Problem,
        cost: Cost,
        row_edges: RowEdges,
    ) -> SparseMatrix {
        rows.iter()
            .map(|next_row| {
                rows.iter()
                    .enumerate()
                    .filter(|(_, prev_row)| {
                        is_valid_transition(prev_row, next_row, graph, problem, row_edges)
                    })
                    .map(|(prev_index, prev_row)| {
                        (
                            prev_index,
                            cost.transition(graph, prev_row, next_row, row_edges),
                        )
                    })
                    .collect()
            })
            .collect()
    }

    /// 次の行を作って集めた遷移が、全部の組を調べた遷移と（順序も含めて）同じか
    #[test_case::test_matrix(
    [2,3,4,5],[true,false],[true,false],[Graph::Grid,Graph::King],[Problem::Dom2,Problem::Fair2]
)]
    fn generated_transitions_equal_all_pairs(
        width: usize,
        left: bool,
        right: bool,
        graph: Graph,
        problem: Problem,
    ) {
        let row_edges = RowEdges { left, right };
        let rows = list_valid_rows(width, graph, problem, row_edges);
        for cost in [Cost::Stones, Cost::Loss] {
            let generated = build_transition_costs(&rows, graph, problem, cost, row_edges);
            let all_pairs =
                build_transition_costs_by_all_pairs(&rows, graph, problem, cost, row_edges);
            assert_eq!(
                generated, all_pairs,
                "width = {width}, left = {left}, right = {right}, graph = {graph}, problem = {problem}, cost = {cost:?}"
            );
            // 密な行列も同じ
            let dense = build_transition_cost_matrix(&rows, graph, problem, cost, row_edges);
            assert_eq!(dense.to_sparse().len(), rows.len());
            for (next_index, transitions) in all_pairs.iter().enumerate() {
                for (prev_index, row) in dense.iter().enumerate() {
                    let expected = transitions
                        .iter()
                        .find(|&&(index, _)| index == prev_index)
                        .map_or(INF, |&(_, value)| value);
                    assert_eq!(row[next_index], expected);
                }
            }
        }
    }

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

        let valid_rows = list_valid_rows(width, Graph::Grid, problem, edges.row());

        let matrix = build_transition_cost_matrix(
            &valid_rows,
            Graph::Grid,
            problem,
            Cost::Loss,
            edges.row(),
        );

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

    /// 盤 board の (row, col) の周りの石の数。盤の四方は端
    fn stones_around(board: &[&Vec<Cell>], row: usize, col: usize, graph: Graph) -> usize {
        let width = board[0].len();
        let all_edges = RowEdges {
            left: true,
            right: true,
        };
        let mut count = 0;
        // 同じ行と、前後の行
        let neighbor_rows_and_offsets = [
            (row.checked_sub(1), graph.adjacent_row_column_offsets()),
            (Some(row), &SAME_ROW_COLUMN_OFFSETS[..]),
            (
                (row + 1 < board.len()).then_some(row + 1),
                graph.adjacent_row_column_offsets(),
            ),
        ];
        for (neighbor_row, offsets) in neighbor_rows_and_offsets {
            let Some(neighbor_row) = neighbor_row else {
                continue;
            };
            for &offset in offsets {
                if let ColumnPlace::Inside(neighbor_col) = column_place(col, offset, width, all_edges)
                    && board[neighbor_row][neighbor_col] == Cell::Stone
                {
                    count += 1;
                }
            }
        }
        count
    }

    /// 四方が端の盤で、遷移できる行の並べ方を全部たどり、
    /// 置き方が問題に合うなら loss から求めた石の数が実際の石の数と一致し、合わないなら INF になることを確かめる
    fn check_all_paths_on_board(width: usize, height: usize, graph: Graph, problem: Problem) {
        let edges = Edges::ALL;
        let valid_rows = list_valid_rows(width, graph, problem, edges.row());
        let matrix: Matrix =
            build_transition_cost_matrix(&valid_rows, graph, problem, Cost::Loss, edges.row());

        search(
            &mut vec![],
            &valid_rows,
            &matrix,
            Cost::Loss,
            height,
            edges,
            graph,
            problem,
        );

        fn check_path(
            path: &[usize],
            valid_rows: &[Vec<Cell>],
            matrix: &Matrix,
            cost: Cost,
            edges: Edges,
            graph: Graph,
            problem: Problem,
        ) {
            let board = path
                .iter()
                .map(|index| &valid_rows[*index])
                .collect::<Vec<&Vec<Cell>>>();

            let mut stone_count = 0;
            let actual_cost = cost.first_row(graph, problem, board[0], &edges)
                + path.windows(2).map(|w| matrix[w[0]][w[1]]).sum::<i64>()
                + cost.last_row_addition(graph, edges.row(), board[board.len() - 1]);

            let mut is_valid = true;
            for row in 0..board.len() {
                for col in 0..board[row].len() {
                    if board[row][col] == Cell::Stone {
                        stone_count += 1;
                    } else {
                        let around_stone_count = stones_around(&board, row, col, graph);
                        if (problem == Problem::Fair2 && around_stone_count != 2)
                            || (problem == Problem::Dom2 && around_stone_count < 2)
                        {
                            is_valid = false;
                        }
                    }
                }
            }

            if is_valid {
                let actual_stone_count =
                    stone_number(graph, cost, actual_cost, board.len(), board[0].len());
                assert_eq!(
                    actual_stone_count, stone_count,
                    "{}, {}, {}, {:?}",
                    actual_stone_count, actual_cost, stone_count, board
                );
            } else {
                assert!(actual_cost >= INF);
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn search(
            path: &mut Vec<usize>,
            valid_rows: &Vec<Vec<Cell>>,
            matrix: &Matrix,
            cost: Cost,
            height: usize,
            edges: Edges,
            graph: Graph,
            problem: Problem,
        ) {
            let valid_cnt = valid_rows.len();
            if path.is_empty() {
                for next_index in 0..valid_cnt {
                    if cost.first_row(graph, problem, &valid_rows[next_index], &edges) == INF {
                        continue;
                    }
                    path.push(next_index);
                    search(path, valid_rows, matrix, cost, height, edges, graph, problem);
                    path.pop();
                }
            } else if path.len() == height {
                check_path(path, valid_rows, matrix, cost, edges, graph, problem);
            } else {
                let prev_index = *path.last().unwrap();
                for next_index in 0..valid_cnt {
                    if matrix[prev_index][next_index] == INF {
                        continue;
                    }
                    path.push(next_index);
                    search(path, valid_rows, matrix, cost, height, edges, graph, problem);
                    path.pop();
                }
            }
        }
    }

    #[test_case::test_matrix(
    [2,3,4,5],[2,3,4,5],[Problem::Dom2,Problem::Fair2]
)]
    fn test_transition_board_cost_grid(width: usize, height: usize, problem: Problem) {
        check_all_paths_on_board(width, height, Graph::Grid, problem);
    }

    /// King は行の候補が多く、2-domination は遷移が密で経路が増えるので、小さい盤だけ
    #[test_case::test_matrix(
    [2,3,4],[2,3,4],[Problem::Dom2,Problem::Fair2]
)]
    fn test_transition_board_cost_king(width: usize, height: usize, problem: Problem) {
        check_all_paths_on_board(width, height, Graph::King, problem);
    }
}
