//! 盤全体（n × m）の最小コストの計算

use crate::{
    dfs::dfs,
    matrix::Vector,
    rules::{Cell, Problem, is_valid_row, list_valid_rows},
    transfer::{INF, build_transition_cost_matrix},
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
pub fn board(rows: usize, cols: usize, problem: Problem, verbose: bool, method: Method) -> i64 {
    match method {
        Method::Dp => dp(rows, cols, problem, verbose),
        Method::Dfs => {
            // 有効な行のリストを取得(左端が端、右端が端)
            let valid_rows = list_valid_rows(cols, true, true);
            // 有効な行のコストを取得
            let row_costs: Vec<i64> = valid_rows
                .iter()
                .map(|row| row.iter().filter(|&&cell| cell == Cell::Stone).count() as i64)
                .collect();

            // コストは盤面全体の石の数
            let (min_cost, min_pattern) = dfs(&valid_rows, rows, problem, |index_list| {
                index_list.iter().map(|&index| row_costs[index]).sum()
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
fn dp(rows: usize, cols: usize, problem: Problem, verbose: bool) -> i64 {
    println!("board: rows = {rows}, cols = {cols}, problem = {problem}");

    // 有効な行のリストを取得(左端が端、右端が端)
    let valid_rows = list_valid_rows(cols, true, true);
    let valid_cnt = valid_rows.len();
    // 有効な行のコストを取得
    let row_costs: Vec<i64> = valid_rows
        .iter()
        .map(|row| row.iter().filter(|&&cell| cell == Cell::Stone).count() as i64)
        .collect();
    println!("valid_count = {valid_cnt}");

    // 遷移コストを取得
    // コストは遷移先の石の数
    // 作られる遷移コスト行列は、遷移先の行が行、遷移元の行が列となる
    let cost_matrix = build_transition_cost_matrix(&valid_rows, problem, true, true, &row_costs);
    let transition_cnt = cost_matrix
        .iter()
        .map(|row| row.iter().filter(|&x| x != &INF).count())
        .sum::<usize>();
    println!(
        "cost_matrix count = {}, valid rate = {:0.2}%",
        transition_cnt,
        transition_cnt as f64 / (valid_cnt * valid_cnt) as f64 * 100.0
    );

    // 遷移コストの計算
    // 1行目の遷移コストを計算
    let mut dp = valid_rows
        .iter()
        .zip(row_costs)
        .map(|(row, cost)| {
            if is_valid_row(row, true, true, true) {
                cost
            } else {
                INF
            }
        })
        .collect::<Vector>();
    // 遷移の経路を保存
    let mut transition_path: Vec<Vec<usize>> = vec![];

    // 残りの行の遷移コストを計算
    for _ in 1..rows {
        // 対象の状態に遷移するためのコストを遷移先ごとに計算
        let mut min_index_list = vec![];
        let mut next_dp = vec![];
        for next_index in 0..valid_cnt {
            let new_dp = &dp + &cost_matrix[next_index];
            let min_index = new_dp.min_index();

            next_dp.push(new_dp[min_index]);
            min_index_list.push(min_index);
        }
        transition_path.push(min_index_list);
        dp = Vector(next_dp);
    }

    // 最後のパスとして使えないパスをINFにする
    for index in 0..valid_cnt {
        if valid_rows[index].contains(&Cell::NeedOne) {
            dp[index] = INF;
        }
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
                board(n, m, Problem::Dom2, true, Method::Dp),
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
                board(n, n, Problem::Fair2, false, Method::Dp),
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
                board(n, m, Problem::Dom2, false, Method::Dfs),
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
                board(n, n, Problem::Fair2, false, Method::Dfs),
                value,
                "n = {n}"
            );
        }
    }
}
