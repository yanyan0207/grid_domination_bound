//! 盤全体（n × m）の最小コストの計算

use std::time::Instant;

use crate::{
    cost::{Cost, stone_number},
    dfs::dfs,
    matrix::{SparseMatrix, Vector},
    rules::{Cell, Edges, Problem, list_valid_rows},
    transfer::build_transition_costs,
};

/// 前の行（先頭行なら `None`）と次の行を受け取り、コストを追加する関数
type AddCostForNextRowFn<'a> = &'a dyn Fn(usize, Option<&[Cell]>, &[Cell]) -> i64;

/// 最小コストを求める方式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// 転送行列（列ごとの DP）
    Dp,
    /// 全探索
    Dfs,
}

/// n × m の盤全体の最小コストの計算。(最小コスト, 石の数) を返す
pub fn board(
    rows: usize,
    cols: usize,
    problem: Problem,
    verbose: bool,
    method: Method,
    cost: Cost,
) -> (i64, i64) {
    let (min_cost, stone_num) = match method {
        Method::Dp => {
            let result = dp(rows, cols, problem, cost, Edges::ALL, None, verbose);
            (result.min_cost, result.stone_num)
        }
        Method::Dfs => {
            if cost == Cost::Loss {
                unimplemented!("DFS は loss に対応していない");
            }
            // 有効な行のリストを取得(四方とも端)
            let edges = Edges::ALL;
            let row_edges = edges.row();
            let valid_rows = list_valid_rows(cols, row_edges);
            // 1 行目のコスト
            let first_row_costs: Vec<i64> = valid_rows
                .iter()
                .map(|row| cost.first_row(row, &edges))
                .collect();

            // コストは 1 行目のコストと、隣り合う行の遷移コストの合計
            let (min_cost, min_pattern) = dfs(&valid_rows, rows, problem, |index_list| {
                let transition_costs: i64 = index_list
                    .windows(2)
                    .map(|pair| {
                        cost.transition(&valid_rows[pair[0]], &valid_rows[pair[1]], row_edges)
                    })
                    .sum();
                first_row_costs[index_list[0]] + transition_costs
            });
            if verbose {
                println!("最小コスト: {min_cost}");
            }
            // 経路の各行で増えたコストと、その行までの累計
            let mut total = 0;
            for (i, &index) in min_pattern.iter().enumerate() {
                let step = if i == 0 {
                    first_row_costs[index]
                } else {
                    cost.transition(
                        &valid_rows[min_pattern[i - 1]],
                        &valid_rows[index],
                        row_edges,
                    )
                };
                total += step;
                print_row(&valid_rows[index], verbose, Some((step, total)));
            }
            (min_cost, stone_number(cost, min_cost, rows, cols))
        }
    };
    if verbose {
        println!("石の数: {stone_num}");
    }
    (min_cost, stone_num)
}

pub struct DpConfig {
    /// 行数
    rows: usize,
    /// 列数
    cols: usize,
    /// 問題の種類
    problem: Problem,
    /// コストの種類
    cost: Cost,
    /// 盤の四方が端かどうか
    edges: Edges,
    /// 有効な行のリスト
    valid_rows: Vec<Vec<Cell>>,
    /// 遷移コスト行列
    cost_matrix: SparseMatrix,
}
/// dp の計算結果
#[derive(Debug)]
pub struct DpResult {
    /// 最小コスト
    pub min_cost: i64,
    /// 石の数
    pub stone_num: i64,
    /// 各行まで進めたときの、状態ごとの最小コスト。dp_history[i] は i + 1 行目まで
    #[allow(unused)]
    pub dp_history: Vec<Vector>,
    /// 最小コストに至る経路
    #[allow(unused)]
    pub min_pattern: Vec<usize>,
}

pub fn dp(
    rows: usize,
    cols: usize,
    problem: Problem,
    cost: Cost,
    edges: Edges,
    add_cost_for_next_row: Option<AddCostForNextRowFn>,
    verbose: bool,
) -> DpResult {
    let config = init_dp(rows, cols, problem, cost, edges, verbose);
    calc_dp(&config, add_cost_for_next_row, verbose)
}

pub fn init_dp(
    rows: usize,
    cols: usize,
    problem: Problem,
    cost: Cost,
    edges: Edges,
    verbose: bool,
) -> DpConfig {
    println!("board: rows = {rows}, cols = {cols}, problem = {problem}");

    let row_edges = edges.row();

    // 有効な行のリストを取得(左端が端、右端が端)
    let valid_rows_start = Instant::now();
    let valid_rows = list_valid_rows(cols, row_edges);
    let valid_cnt = valid_rows.len();
    println!("valid_count = {valid_cnt}");
    if verbose {
        println!(
            "valid_rows time = {:.3}s",
            valid_rows_start.elapsed().as_secs_f64()
        );
    }

    // 遷移コストを取得
    // コストは --cost で選んだもの
    // 作られる遷移コスト行列は、遷移先の行が行、遷移元の行が列となる
    let cost_matrix_start = Instant::now();
    let cost_matrix = build_transition_costs(&valid_rows, problem, cost, row_edges);
    let cost_matrix_seconds = cost_matrix_start.elapsed().as_secs_f64();
    let transition_cnt = cost_matrix.iter().map(|row| row.len()).sum::<usize>();

    if verbose {
        println!(
            "cost_matrix count = {}, valid rate = {:0.2}%",
            transition_cnt,
            transition_cnt as f64 / (valid_cnt * valid_cnt) as f64 * 100.0
        );
        println!("cost_matrix time = {cost_matrix_seconds:.3}s");
    }
    DpConfig {
        rows,
        cols,
        problem,
        cost,
        edges,
        valid_rows,
        cost_matrix,
    }
}

/// 転送行列（DP）で最小コストを求める。edges は盤の四方が端かどうか
pub fn calc_dp(
    dp_config: &DpConfig,
    add_cost_for_next_row: Option<AddCostForNextRowFn>,
    verbose: bool,
) -> DpResult {
    let DpConfig {
        rows,
        cols,
        problem: _problem,
        cost,
        edges,
        valid_rows,
        cost_matrix,
    } = dp_config;

    let valid_cnt = valid_rows.len();

    // 遷移コストの計算
    // 1行目の遷移コストを計算
    let dp = valid_rows
        .iter()
        .map(|row| {
            cost.first_row(row, edges) + add_cost_for_next_row.map_or(0, |f| f(0, None, row))
        })
        .collect::<Vector>();
    let mut dp_history = vec![dp.clone()];

    // 遷移の経路を保存
    let mut transition_path: Vec<Vec<usize>> = vec![];

    // 残りの行の遷移コストを計算
    let start = Instant::now();
    for row in 1..*rows {
        // 対象の状態に遷移するためのコストを遷移先ごとに計算
        let mut min_index_list = Vec::with_capacity(valid_cnt);
        let mut next_dp = Vec::with_capacity(valid_cnt);
        let dp = dp_history.last().unwrap();
        for (next_index, costs) in cost_matrix.iter().enumerate() {
            // 現在の行のインデックス next_index に対して、遷移元の行からの最小コストを計算
            let min_costs = costs
                .iter()
                .map(|(prev_index, val)| {
                    (
                        prev_index,
                        val + dp[*prev_index]
                            + add_cost_for_next_row.map_or(0, |f| {
                                f(row, Some(&valid_rows[*prev_index]), &valid_rows[next_index])
                            }),
                    )
                })
                .min_by_key(|&(_prev_index, val)| val)
                .unwrap();
            min_index_list.push(*min_costs.0);
            next_dp.push(min_costs.1);
        }
        transition_path.push(min_index_list);
        dp_history.push(Vector(next_dp));

        // 進捗を標準エラー出力に、同じ行を上書きして表示する
        if verbose {
            eprint!(
                "\r{row}/{} 行 ({:.1}s)",
                rows - 1,
                start.elapsed().as_secs_f64()
            );
        }
    }
    if verbose {
        eprintln!();
    }

    // 最後のパスとして使えないパスをINFにする
    let dp = dp_history.last_mut().unwrap();
    if edges.bottom {
        for (i, valid_row) in valid_rows.iter().enumerate() {
            dp[i] += cost.last_row_addition(edges.row(), valid_row);
        }
    }

    // 最小コストとその経路を出力
    let last_min_index = dp.min_index();
    let min_cost = dp[last_min_index];
    if verbose {
        println!("最小コストのインデックス: {last_min_index}");
        println!("最小コスト: {min_cost}");
    }

    // 逆順にたどる
    let mut min_pattern = vec![last_min_index];
    let mut work = last_min_index;
    for i in (0..transition_path.len()).rev() {
        work = transition_path[i][work];
        min_pattern.push(work);
    }
    min_pattern.reverse();
    // 経路の各行で増えたコストと、その行までの累計（最後の行は最後の行の追加分を含む）
    let mut previous_total = 0;
    for (i, &index) in min_pattern.iter().enumerate() {
        let total = dp_history[i][index];
        print_row(
            &valid_rows[index],
            verbose,
            Some((total - previous_total, total)),
        );
        previous_total = total;
    }

    DpResult {
        min_cost,
        stone_num: stone_number(*cost, min_cost, *rows, *cols),
        dp_history,
        min_pattern,
    }
}

/// 行を 1 行表示する。verbose のときは step_cost =（その行で増えたコスト, 累計）も表示する
fn print_row(row: &[Cell], verbose: bool, step_cost: Option<(i64, i64)>) {
    if !verbose {
        return;
    }
    let line: String = row
        .iter()
        .map(|cell| match cell {
            Cell::Stone => '#',
            Cell::Ok => '.',
            Cell::NeedOne if verbose => '?',
            Cell::NeedOne => '.',
        })
        .collect();
    match step_cost {
        Some((step, total)) if verbose => println!("{line}  +{step} (累計 {total})"),
        _ => println!("{line}"),
    }
}

#[cfg(test)]
mod tests {
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
    fn check_dom2(method: Method, cost: Cost) {
        for (n, m, stones) in DOM2_EXPECTED {
            assert_eq!(
                board(n, m, Problem::Dom2, false, method, cost).1,
                stones,
                "n = {n}, m = {m}"
            );
        }
    }

    /// 2-fair の石の数が、OEIS の値と一致するか
    fn check_fair2(method: Method, cost: Cost) {
        for (n, stones) in FAIR2_EXPECTED {
            assert_eq!(
                board(n, n, Problem::Fair2, false, method, cost).1,
                stones,
                "n = {n}"
            );
        }
    }

    #[test]
    fn dp_stones_dom2() {
        check_dom2(Method::Dp, Cost::Stones);
    }

    #[test]
    fn dp_stones_fair2() {
        check_fair2(Method::Dp, Cost::Stones);
    }

    #[test]
    fn dfs_stones_dom2() {
        check_dom2(Method::Dfs, Cost::Stones);
    }

    #[test]
    fn dfs_stones_fair2() {
        check_fair2(Method::Dfs, Cost::Stones);
    }

    #[test]
    fn dp_loss_dom2() {
        check_dom2(Method::Dp, Cost::Loss);
    }

    #[test]
    fn dp_loss_fair2() {
        check_fair2(Method::Dp, Cost::Loss);
    }
}
