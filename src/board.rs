//! 盤全体（n × m）の最小コストの計算

use std::time::Instant;

use crate::{
    cost::Cost,
    dfs::dfs,
    matrix::Vector,
    rules::{Cell, Problem, list_valid_rows},
    transfer::build_transition_costs,
};

/// 最小コストを求める方式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// 転送行列（列ごとの DP）
    Dp,
    /// 全探索
    Dfs,
}

/// n × m の盤全体の最小コストの計算。最小コストを返す
pub fn board(
    rows: usize,
    cols: usize,
    problem: Problem,
    verbose: bool,
    method: Method,
    cost: Cost,
) -> i64 {
    if cost == Cost::Loss {
        todo!("loss は未実装");
    }
    match method {
        Method::Dp => dp(rows, cols, problem, cost, verbose),
        Method::Dfs => {
            // 有効な行のリストを取得(左端が端、右端が端)
            let valid_rows = list_valid_rows(cols, true, true);
            // 1 行目のコスト
            let first_row_costs: Vec<i64> = valid_rows
                .iter()
                .map(|row| cost.first_row(row, true, true, true))
                .collect();

            // コストは 1 行目のコストと、隣り合う行の遷移コストの合計
            let (min_cost, min_pattern) = dfs(&valid_rows, rows, problem, |index_list| {
                let transition_costs: i64 = index_list
                    .windows(2)
                    .map(|pair| {
                        cost.transition(&valid_rows[pair[0]], &valid_rows[pair[1]], true, true)
                    })
                    .sum();
                first_row_costs[index_list[0]] + transition_costs
            });
            println!("最小コスト: {min_cost}");
            for &index in &min_pattern {
                print_row(&valid_rows[index], verbose);
            }
            min_cost
        }
    }
}

/// 転送行列（DP）で最小コストを求める
fn dp(rows: usize, cols: usize, problem: Problem, cost: Cost, verbose: bool) -> i64 {
    println!("board: rows = {rows}, cols = {cols}, problem = {problem}");

    let left_is_edge = true;
    let right_is_edge = true;
    let top_is_edge = true;

    // 有効な行のリストを取得(左端が端、右端が端)
    let valid_rows_start = Instant::now();
    let valid_rows = list_valid_rows(cols, left_is_edge, right_is_edge);
    let valid_cnt = valid_rows.len();
    println!(
        "valid_count = {valid_cnt} ({:.3}s)",
        valid_rows_start.elapsed().as_secs_f64()
    );

    // 遷移コストを取得
    // コストは --cost で選んだもの
    // 作られる遷移コスト行列は、遷移先の行が行、遷移元の行が列となる
    let cost_matrix_start = Instant::now();
    let cost_matrix =
        build_transition_costs(&valid_rows, problem, cost, left_is_edge, right_is_edge);
    let cost_matrix_seconds = cost_matrix_start.elapsed().as_secs_f64();
    let transition_cnt = cost_matrix.iter().map(|row| row.len()).sum::<usize>();
    println!(
        "cost_matrix count = {}, valid rate = {:0.2}% ({:.3}s)",
        transition_cnt,
        transition_cnt as f64 / (valid_cnt * valid_cnt) as f64 * 100.0,
        cost_matrix_seconds
    );

    // 遷移コストの計算
    // 1行目の遷移コストを計算
    let dp = valid_rows
        .iter()
        .map(|row| cost.first_row(row, left_is_edge, right_is_edge, top_is_edge))
        .collect::<Vector>();
    let mut dp_history = vec![dp.clone()];

    // 遷移の経路を保存
    let mut transition_path: Vec<Vec<usize>> = vec![];

    // 残りの行の遷移コストを計算
    let start = Instant::now();
    for row in 1..rows {
        // 対象の状態に遷移するためのコストを遷移先ごとに計算
        let mut min_index_list = Vec::with_capacity(valid_cnt);
        let mut next_dp = Vec::with_capacity(valid_cnt);
        let dp = dp_history.last().unwrap();
        for costs in cost_matrix.iter() {
            // 現在の行のインデックス next_index に対して、遷移元の行からの最小コストを計算
            let min_costs = costs
                .iter()
                .map(|(prev_index, val)| (prev_index, val + dp[*prev_index]))
                .min_by_key(|&(_prev_index, val)| val)
                .unwrap();
            min_index_list.push(*min_costs.0);
            next_dp.push(min_costs.1);
        }
        transition_path.push(min_index_list);
        dp_history.push(Vector(next_dp));

        // 進捗を標準エラー出力に、同じ行を上書きして表示する
        eprint!(
            "\r{row}/{} 行 ({:.1}s)",
            rows - 1,
            start.elapsed().as_secs_f64()
        );
    }
    eprintln!();

    // 最後のパスとして使えないパスをINFにする
    let dp = dp_history.last_mut().unwrap();
    for (i, valid_row) in valid_rows.iter().enumerate() {
        dp[i] += cost.last_row_addition(left_is_edge, right_is_edge, valid_row);
    }

    // 最小コストとその経路を出力
    let last_min_index = dp.min_index();
    let min_cost = dp[last_min_index];
    println!("最小コストのインデックス: {last_min_index}");
    println!("最小コスト: {min_cost}");

    // 逆順にたどる
    let mut min_pattern = vec![last_min_index];
    let mut work = last_min_index;
    for i in (0..transition_path.len()).rev() {
        work = transition_path[i][work];
        min_pattern.push(work);
    }
    min_pattern.reverse();
    for &index in &min_pattern {
        print_row(&valid_rows[index], verbose);
    }
    min_cost
}

fn print_row(row: &[Cell], verbose: bool) {
    let line: String = row
        .iter()
        .map(|cell| match cell {
            Cell::Stone => '#',
            Cell::Ok => '.',
            Cell::NeedOne if verbose => '?',
            Cell::NeedOne => '.',
        })
        .collect();
    println!("{line}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dom2_matches_talon_tool() {
        // arXiv:1810.12896 の著者のツール（Source/release.exe）の出力。(行数 n, 列数 m, 最小の石の数)。幅 1 は扱わない
        let expected = [
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
        for (n, m, value) in expected {
            assert_eq!(
                board(n, m, Problem::Dom2, true, Method::Dp, Cost::Stones),
                value,
                "n = {n}, m = {m}"
            );
        }
    }

    #[test]
    fn fair2_matches_oeis_a344719() {
        // OEIS A344719（n × n の 2-fair の最小の石の数）の a(2)〜a(6)。幅 1 は扱わない
        let expected = [2, 5, 8, 11, 17];
        for (i, value) in expected.into_iter().enumerate() {
            let n = i + 2;
            assert_eq!(
                board(n, n, Problem::Fair2, false, Method::Dp, Cost::Stones),
                value,
                "n = {n}"
            );
        }
    }

    #[test]
    fn dfs_dom2_matches_talon_tool() {
        // arXiv:1810.12896 の著者のツール（Source/release.exe）の出力。(行数 n, 列数 m, 最小の石の数)。幅 1 は扱わない
        let expected = [
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
        for (n, m, value) in expected {
            assert_eq!(
                board(n, m, Problem::Dom2, false, Method::Dfs, Cost::Stones),
                value,
                "n = {n}, m = {m}"
            );
        }
    }

    #[test]
    fn dfs_fair2_matches_oeis_a344719() {
        // OEIS A344719（n × n の 2-fair の最小の石の数）の a(2)〜a(6)。幅 1 は扱わない
        let expected = [2, 5, 8, 11, 17];
        for (i, value) in expected.into_iter().enumerate() {
            let n = i + 2;
            assert_eq!(
                board(n, n, Problem::Fair2, false, Method::Dfs, Cost::Stones),
                value,
                "n = {n}"
            );
        }
    }
}
