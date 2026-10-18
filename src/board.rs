//! 盤全体（n × m）の最小コストの計算

use crate::{
    cost::{Cost, stones_coefficient},
    dp::{calc_dp, init_dp, print_dp_result},
    rules::{Edges, Graph, Problem},
};

/// n × m の盤全体の最小コストの計算。(最小コスト, 石の数) を返す
pub fn board(
    rows: usize,
    cols: usize,
    graph: Graph,
    problem: Problem,
    verbose: bool,
    cost: Cost,
) -> (i64, i64) {
    let dp_config = init_dp(rows, cols, graph, problem, cost, Edges::ALL, verbose);
    let result = calc_dp(&dp_config, None, verbose);
    print_dp_result(&dp_config, &result);
    let (min_cost, stone_num) = (result.min_cost, result.stone_num);
    // 盤全体の置き方では (次数 + 2)|D| = 2nm + loss がちょうど成り立つので、石の数から loss を求める
    // （--cost stones のときは DP が loss を計算しないため）
    let loss = stones_coefficient(graph) * stone_num - 2 * (rows * cols) as i64;
    println!("石の数: {stone_num}");
    println!("loss: {loss}");
    (min_cost, stone_num)
}

#[cfg(test)]
mod tests {
    use crate::rules::{ColumnPlace, RowEdges, SAME_ROW_COLUMN_OFFSETS, column_place};

    use super::*;

    /// arXiv:1810.12896 の著者のツール（Source/release.exe）の出力。(行数 n, 列数 m, 最小の石の数)。幅 1 は扱わない
    const DOM2_EXPECTED: [(usize, usize, i64); 12] = [
        (2, 2, 2),
        (2, 3, 3),
        (2, 4, 4),
        (2, 5, 5),
        (2, 6, 6),
        (3, 3, 4),
        (3, 4, 6),
        (3, 5, 7),
        (3, 6, 8),
        (4, 4, 8),
        (4, 5, 10),
        (4, 6, 12),
    ];

    /// OEIS A344719（n × n の 2-fair の最小の石の数）の a(2)〜a(6)。幅 1 は扱わない
    const FAIR2_EXPECTED: [(usize, i64); 5] = [(2, 2), (3, 5), (4, 8), (5, 11), (6, 17)];

    /// 2-domination の石の数が、著者のツールの値と一致するか
    fn check_dom2(cost: Cost) {
        for (n, m, stones) in DOM2_EXPECTED {
            assert_eq!(
                board(n, m, Graph::Grid, Problem::Dom2, false, cost).1,
                stones,
                "n = {n}, m = {m}"
            );
        }
    }

    /// 2-fair の石の数が、OEIS の値と一致するか
    fn check_fair2(cost: Cost) {
        for (n, stones) in FAIR2_EXPECTED {
            assert_eq!(
                board(n, n, Graph::Grid, Problem::Fair2, false, cost).1,
                stones,
                "n = {n}"
            );
        }
    }

    #[test]
    fn dp_stones_dom2() {
        check_dom2(Cost::Stones);
    }

    #[test]
    fn dp_stones_fair2() {
        check_fair2(Cost::Stones);
    }

    #[test]
    fn dp_loss_dom2() {
        check_dom2(Cost::Loss);
    }

    #[test]
    fn dp_loss_fair2() {
        check_fair2(Cost::Loss);
    }

    /// rows × cols の盤の全部の置き方を調べて、最小の石の数を直接求める（DP と独立な検算）
    fn min_stones_by_brute_force(rows: usize, cols: usize, graph: Graph, problem: Problem) -> i64 {
        let all_edges = RowEdges {
            left: true,
            right: true,
        };
        let cell_count = rows * cols;
        let mut min_stones = i64::MAX;
        for placement in 0u64..(1 << cell_count) {
            let is_stone = |row: usize, col: usize| (placement >> (row * cols + col)) & 1 == 1;
            let mut all_cells_fit = true;
            'cells: for row in 0..rows {
                for col in 0..cols {
                    if is_stone(row, col) {
                        continue;
                    }
                    let mut stones_around = 0;
                    let neighbor_rows_and_offsets = [
                        (row.checked_sub(1), graph.adjacent_row_column_offsets()),
                        (Some(row), &SAME_ROW_COLUMN_OFFSETS[..]),
                        (
                            (row + 1 < rows).then_some(row + 1),
                            graph.adjacent_row_column_offsets(),
                        ),
                    ];
                    for (neighbor_row, offsets) in neighbor_rows_and_offsets {
                        let Some(neighbor_row) = neighbor_row else {
                            continue;
                        };
                        for &offset in offsets {
                            if let ColumnPlace::Inside(neighbor_col) =
                                column_place(col, offset, cols, all_edges)
                                && is_stone(neighbor_row, neighbor_col)
                            {
                                stones_around += 1;
                            }
                        }
                    }
                    let fits = match problem {
                        Problem::Fair2 => stones_around == 2,
                        Problem::Dom2 => stones_around >= 2,
                    };
                    if !fits {
                        all_cells_fit = false;
                        break 'cells;
                    }
                }
            }
            if all_cells_fit {
                min_stones = min_stones.min(placement.count_ones() as i64);
            }
        }
        min_stones
    }

    /// 小さい盤で、DP の石の数が全部の置き方を調べた最小と一致するか（Grid / King × 2-domination / 2-fair × 石の数 / loss）
    #[test_case::test_matrix(
    [(1, 1), (1, 4), (2, 2), (2, 3), (3, 2), (3, 3), (2, 4), (4, 2), (3, 4), (4, 3), (4, 4), (3, 5), (5, 3)],
    [Graph::Grid, Graph::King],
    [Problem::Dom2, Problem::Fair2]
)]
    fn dp_matches_brute_force(size: (usize, usize), graph: Graph, problem: Problem) {
        let (rows, cols) = size;
        let expected = min_stones_by_brute_force(rows, cols, graph, problem);
        for cost in [Cost::Stones, Cost::Loss] {
            assert_eq!(
                board(rows, cols, graph, problem, false, cost).1,
                expected,
                "rows = {rows}, cols = {cols}, graph = {graph}, problem = {problem}, cost = {cost:?}"
            );
        }
    }

    /// King の 2-fair が Hung（Mathematics 14(6):947, 2026）の値と一致するか。
    /// 1×n は ⌊n/2⌋ + 1（道と同じなので Grid も同じ）、2×n は 2⌈n/3⌉、5×5 は 8、6×6 は全部の頂点
    #[test]
    fn king_fair2_matches_paper() {
        for n in 1..=10 {
            for graph in [Graph::Grid, Graph::King] {
                assert_eq!(
                    board(n, 1, graph, Problem::Fair2, false, Cost::Stones).1,
                    (n / 2 + 1) as i64,
                    "1 × {n}, graph = {graph}"
                );
            }
        }
        for n in 1..=12 {
            assert_eq!(
                board(n, 2, Graph::King, Problem::Fair2, false, Cost::Stones).1,
                (2 * n.div_ceil(3)) as i64,
                "2 × {n}"
            );
        }
        for cost in [Cost::Stones, Cost::Loss] {
            assert_eq!(board(5, 5, Graph::King, Problem::Fair2, false, cost).1, 8);
            assert_eq!(board(6, 6, Graph::King, Problem::Fair2, false, cost).1, 36);
        }
    }

    /// 7×7 は全部の頂点、8×8 は 18。遅いので cargo test --release -- --ignored で実行する
    #[test]
    #[ignore]
    fn king_fair2_matches_paper_large() {
        assert_eq!(
            board(7, 7, Graph::King, Problem::Fair2, false, Cost::Stones).1,
            49
        );
        assert_eq!(
            board(8, 8, Graph::King, Problem::Fair2, false, Cost::Stones).1,
            18
        );
    }
}
