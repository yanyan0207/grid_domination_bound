//! 行を積む DP の周期性の確認
//!
//! 幅 width の行を 1 行ずつ積む DP で、始まりのベクトルの組 V_0 を r 行進めたものを V_r とする。
//! ある r₀ から先は k 行進めるごとに全部のベクトルの全成分がちょうど p 増える
//! （V_(r₀+k) = V_r₀ + p）かを調べる。一度成り立てば、両辺を同じ遷移で 1 行進めると
//! r ≥ r₀ のすべてで成り立つので、1 回の一致を見つければ十分
//!
//! 始まりの行は上端（top）で決まる
//! - 上端が盤の端: 上端の行のコストの 1 本のベクトルから始める（盤の DP と同じ）
//! - 上端が開いている: 状態ごとに、その状態だけ 0 のベクトルから始める。
//!   状態 s から始めたベクトルを r 行進めたものは、遷移行列の r 乗の行 s なので、行列の周期と同じ

use crate::{
    cost::Cost,
    dp::{init_dp, next_row_dp},
    matrix::{SparseMatrix, Vector},
    rules::{Edges, Problem},
    transfer::INF,
};

/// V_(start_rows + period) = V_start_rows + increase
#[derive(Debug, PartialEq, Eq)]
pub struct Period {
    /// r₀: 始まりの行から進めた行数
    pub start_rows: usize,
    /// k
    pub period: usize,
    /// p
    pub increase: i64,
}

/// 幅 width の行を積む DP の周期を調べて表示する
///
/// left_is_edge / right_is_edge は行の左端・右端の外側が盤の端か、
/// top_is_edge は始まりの行の外側が盤の端か
pub fn row_period(
    width: usize,
    problem: Problem,
    cost: Cost,
    left_is_edge: bool,
    right_is_edge: bool,
    top_is_edge: bool,
    max_rows: usize,
) {
    // 下端は周期の判定に関係しないので、盤の端としない
    let edges = Edges {
        top: top_is_edge,
        bottom: false,
        left: left_is_edge,
        right: right_is_edge,
    };
    let dp_config = init_dp(max_rows, width, problem, cost, edges, false);
    let state_count = dp_config.valid_rows.len();

    let start_vectors: Vec<Vector> = if top_is_edge {
        vec![
            dp_config
                .valid_rows
                .iter()
                .map(|row| cost.first_row(row, &edges))
                .collect(),
        ]
    } else {
        (0..state_count)
            .map(|state| {
                (0..state_count)
                    .map(|i| if i == state { 0 } else { INF })
                    .collect()
            })
            .collect()
    };

    println!(
        "幅 {width}、problem = {problem}、cost = {cost:?}、左端 {}、右端 {}、始まりの行 {}、状態の数 {state_count}、始まりのベクトルの数 {}",
        side_text(left_is_edge),
        side_text(right_is_edge),
        side_text(top_is_edge),
        start_vectors.len()
    );
    match find_period(start_vectors, &dp_config.cost_matrix, max_rows) {
        Some(Period {
            start_rows,
            period,
            increase,
        }) => {
            println!("r0 = {start_rows}, k = {period}, p = {increase}");
            println!("1 行あたりの増え方 = {}", increase as f64 / period as f64);
        }
        None => println!("{max_rows} 行進めるまで周期なし"),
    }
}

/// 辺の外側の表示。盤の端なら「edge」、盤の内側なら「open」
fn side_text(is_edge: bool) -> &'static str {
    if is_edge { "edge" } else { "open" }
}

/// 始まりのベクトルの組を 1 行ずつ進め、前の組の全成分に同じ値を足したものと一致する最初の組を返す
///
/// 進めた行数 r が小さい順に、それより前の組（進めた行数が小さい順）と比べる。
/// max_rows 行進めても見つからなければ None
pub fn find_period(
    start_vectors: Vec<Vector>,
    cost_matrix: &SparseMatrix,
    max_rows: usize,
) -> Option<Period> {
    // history[r] は r 行進めたベクトルの組
    let mut history = vec![start_vectors];
    for rows in 1..=max_rows {
        let next: Vec<Vector> = history
            .last()
            .unwrap()
            .iter()
            .map(|vector| next_row_dp(vector, cost_matrix, |_, _| 0).0)
            .collect();
        for (earlier_rows, earlier) in history.iter().enumerate() {
            if let Some(increase) = constant_difference(earlier, &next) {
                return Some(Period {
                    start_rows: earlier_rows,
                    period: rows - earlier_rows,
                    increase,
                });
            }
        }
        history.push(next);
    }
    None
}

/// later が earlier の全部のベクトルの全成分に同じ値を足したものなら、その値を返す。
/// INF 以上（つながらない）の位置も同じであること
fn constant_difference(earlier: &[Vector], later: &[Vector]) -> Option<i64> {
    let mut difference = None;
    for (earlier_vector, later_vector) in earlier.iter().zip(later.iter()) {
        for (&earlier_cost, &later_cost) in earlier_vector.iter().zip(later_vector.iter()) {
            match (earlier_cost >= INF, later_cost >= INF) {
                // どちらもつながらない
                (true, true) => continue,
                (false, false) => {
                    let increase = later_cost - earlier_cost;
                    if *difference.get_or_insert(increase) != increase {
                        return None;
                    }
                }
                // 片方だけつながる
                _ => return None,
            }
        }
    }
    difference
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 状態ごとの単位ベクトル（その状態だけ 0）
    fn unit_vectors(state_count: usize) -> Vec<Vector> {
        (0..state_count)
            .map(|state| {
                (0..state_count)
                    .map(|i| if i == state { 0 } else { INF })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn finds_period_one() {
        // 状態 1 つ、遷移コスト 2。1 行進めるごとに 2 増える
        let cost_matrix: SparseMatrix = vec![vec![(0, 2)]];
        assert_eq!(
            find_period(unit_vectors(1), &cost_matrix, 10),
            Some(Period {
                start_rows: 0,
                period: 1,
                increase: 2
            })
        );
    }

    #[test]
    fn finds_period_two_with_inf() {
        // 2 つの状態を行き来するだけ（コスト 1）。0 行と 1 行では INF の位置が違い、2 行進めると 2 増える
        let cost_matrix: SparseMatrix = vec![vec![(1, 1)], vec![(0, 1)]];
        assert_eq!(
            find_period(unit_vectors(2), &cost_matrix, 10),
            Some(Period {
                start_rows: 0,
                period: 2,
                increase: 2
            })
        );
    }

    #[test]
    fn no_period_when_growth_differs() {
        // 状態 0 は 1 ずつ、状態 1 は 2 ずつ増え、どの 2 つの組の差も成分ごとに違う
        let cost_matrix: SparseMatrix = vec![vec![(0, 1)], vec![(1, 2)]];
        assert_eq!(find_period(unit_vectors(2), &cost_matrix, 10), None);
    }
}
