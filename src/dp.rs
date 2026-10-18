//! 転送行列（行ごとの DP）で最小コストを求める

use std::{fmt, time::Instant};

use crate::{
    cost::{Cost, stone_number},
    display::print_row_with_cost,
    matrix::{SparseMatrix, Vector},
    rules::{Cell, Edges, Graph, Problem, list_valid_rows},
    transfer::{INF, build_transition_costs},
};

/// 前の行（先頭行なら `None`）と次の行を受け取り、コストを追加する関数
pub type AddCostForNextRowFn<'a> = &'a dyn Fn(usize, Option<&[Cell]>, &[Cell]) -> i64;

pub struct DpConfig {
    /// 行数
    pub rows: usize,
    /// 列数
    pub cols: usize,
    /// グラフの種類
    pub graph: Graph,
    /// 問題の種類
    pub problem: Problem,
    /// コストの種類
    pub cost: Cost,
    /// 盤の四方が端かどうか
    pub edges: Edges,
    /// 有効な行のリスト
    pub valid_rows: Vec<Vec<Cell>>,
    /// 遷移コスト行列
    pub cost_matrix: SparseMatrix,
}

// 表示用：{}
impl fmt::Display for DpConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "DpConfig {{")?;
        writeln!(f, "\trows: {}", self.rows)?;
        writeln!(f, "\tcols: {}", self.cols)?;
        writeln!(f, "\tgraph: {:?}", self.graph)?;
        writeln!(f, "\tproblem: {:?}", self.problem)?;
        writeln!(f, "\tcost: {:?}", self.cost)?;
        writeln!(f, "\tedges: {:?}", self.edges)?;
        writeln!(
            f,
            "\tvalid_rows count: {} x {}",
            self.valid_rows[0].len(),
            self.valid_rows.len()
        )?;
        writeln!(
            f,
            "\tcost_matrix count: {} x {}",
            self.cost_matrix.len(),
            self.cost_matrix[0].len()
        )?;
        writeln!(f, "}}")?;
        Ok(())
    }
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
    /// 遷移パス
    pub transition_path: Vec<Vec<usize>>,
}

pub fn init_dp(
    rows: usize,
    cols: usize,
    graph: Graph,
    problem: Problem,
    cost: Cost,
    edges: Edges,
    verbose: bool,
) -> DpConfig {
    if verbose {
        println!("init_dp start");
        println!("board: rows = {rows}, cols = {cols}, graph = {graph}, problem = {problem}");
    }

    let row_edges = edges.row();

    // 有効な行のリストを取得(左端が端、右端が端)
    let valid_rows_start = Instant::now();
    let valid_rows = list_valid_rows(cols, graph, problem, row_edges);
    let valid_cnt = valid_rows.len();
    if verbose {
        println!("valid_count = {valid_cnt}");
        println!(
            "valid_rows time = {:.3}s",
            valid_rows_start.elapsed().as_secs_f64()
        );
    }

    // 遷移コストを取得
    // コストは --cost で選んだもの
    // 作られる遷移コスト行列は、遷移先の行が行、遷移元の行が列となる
    let cost_matrix_start = Instant::now();
    let cost_matrix = build_transition_costs(&valid_rows, graph, problem, cost, row_edges);
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
        graph,
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
        graph,
        problem,
        cost,
        edges,
        valid_rows,
        cost_matrix,
    } = dp_config;

    // 遷移コストの計算
    // 1行目の遷移コストを計算
    let dp = valid_rows
        .iter()
        .map(|row| {
            cost.first_row(*graph, *problem, row, edges)
                + add_cost_for_next_row.map_or(0, |f| f(0, None, row))
        })
        .collect::<Vector>();
    let mut dp_history = vec![dp.clone()];

    // 遷移の経路を保存
    let mut transition_path: Vec<Vec<usize>> = vec![];

    // 残りの行の遷移コストを計算
    let start = Instant::now();
    for row in 1..*rows {
        let (next_dp, min_index_list) = next_row_dp(
            dp_history.last().unwrap(),
            cost_matrix,
            |prev_index, next_index| {
                add_cost_for_next_row.map_or(0, |f| {
                    f(row, Some(&valid_rows[prev_index]), &valid_rows[next_index])
                })
            },
        );
        transition_path.push(min_index_list);
        dp_history.push(next_dp);

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
            dp[i] += cost.last_row_addition(*graph, edges.row(), valid_row);
        }
    }

    // 最小コストとその経路
    let last_min_index = dp.min_index();
    let min_cost = dp[last_min_index];

    // 逆順にたどる
    let mut min_pattern = vec![last_min_index];
    let mut work = last_min_index;
    for i in (0..transition_path.len()).rev() {
        work = transition_path[i][work];
        min_pattern.push(work);
    }
    min_pattern.reverse();

    DpResult {
        min_cost,
        stone_num: stone_number(*graph, *cost, min_cost, *rows, *cols),
        dp_history,
        min_pattern,
        transition_path,
    }
}

/// DP を 1 行進める。dp[i] は、今の行が状態 i になるまでの最小コスト
///
/// cost_matrix[next] は (遷移元 prev, 遷移コスト) の一覧。add_cost(prev, next) は遷移コストに足す追加のコスト。
/// 戻り値は (次の行の dp, 次の行の状態ごとの、最小になったときの遷移元)
pub fn next_row_dp(
    dp: &Vector,
    cost_matrix: &SparseMatrix,
    add_cost: impl Fn(usize, usize) -> i64,
) -> (Vector, Vec<usize>) {
    let mut next_dp = Vec::with_capacity(cost_matrix.len());
    let mut min_index_list = Vec::with_capacity(cost_matrix.len());
    for (next_index, costs) in cost_matrix.iter().enumerate() {
        // 次の行の状態 next_index に、遷移元の行から来る最小コスト。どの行からも来られなければ INF
        let (min_prev_index, min_cost) = costs
            .iter()
            .map(|&(prev_index, cost)| {
                (
                    prev_index,
                    cost + dp[prev_index] + add_cost(prev_index, next_index),
                )
            })
            .min_by_key(|&(_, cost)| cost)
            .unwrap_or((0, INF));
        min_index_list.push(min_prev_index);
        next_dp.push(min_cost);
    }
    (Vector(next_dp), min_index_list)
}

/// calc_dp の結果（最小コストと、それに至る盤面）を表示する
pub fn print_dp_result(dp_config: &DpConfig, result: &DpResult) {
    let last_min_index = *result.min_pattern.last().unwrap();
    println!("最小コストのインデックス: {last_min_index}");
    println!("最小コスト: {}", result.min_cost);

    // 経路の各行で増えたコストと、その行までの累計（最後の行は最後の行の追加分を含む）
    let mut previous_total = 0;
    for (i, &index) in result.min_pattern.iter().enumerate() {
        let total = result.dp_history[i][index];
        print_row_with_cost(
            &dp_config.valid_rows[index],
            true,
            Some((total - previous_total, total)),
        );
        previous_total = total;
    }
}
