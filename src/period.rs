//! 帯の行列の周期性の確認
//!
//! 帯の行列 B の累乗が、ある r₀ から先は k 回掛けるごとに全成分がちょうど p 増える
//! （B^(r₀+k) = B^r₀ + p）かを調べる。一度成り立てば、両辺に B を掛けると
//! r ≥ r₀ のすべてで成り立つので、1 回の一致を見つければ十分

use crate::{
    cost::Cost,
    matrix::{Matrix, compose},
    rules::{Problem, RowEdges, list_valid_rows},
    transfer::{INF, build_transition_cost_matrix},
};

/// B^(start_power + period) = B^start_power + increase
#[derive(Debug, PartialEq, Eq)]
pub struct BandPeriod {
    /// r₀
    pub start_power: usize,
    /// k
    pub period: usize,
    /// p
    pub increase: i64,
}

/// 幅 band_width の帯の行列を作り、周期を調べて表示する
///
/// 帯の行列は、左端が盤の端で右端が開いた幅 band_width の行の、loss の遷移行列
pub fn band_period(band_width: usize, problem: Problem, max_power: usize) {
    let edges = RowEdges {
        left: true,
        right: false,
    };
    let band_rows = list_valid_rows(band_width, edges);
    let band_matrix = build_transition_cost_matrix(&band_rows, problem, Cost::Loss, edges);
    println!(
        "帯の幅 {band_width}、problem = {problem}、帯の状態の数 {}",
        band_rows.len()
    );

    match find_band_period(&band_matrix, max_power) {
        Some(BandPeriod {
            start_power,
            period,
            increase,
        }) => {
            println!("r0 = {start_power}, k = {period}, p = {increase}");
            println!("帯 1 行あたりの loss = {}", increase as f64 / period as f64);
        }
        None => println!("B^{max_power} まで周期なし"),
    }
}

/// B, B², … を順に作り、前の累乗の全成分に同じ値を足したものと一致する最初の組を返す
///
/// 累乗 B^r が小さい順に、それより前の B^s（s が小さい順）と比べる。
/// max_power まで掛けても見つからなければ None
pub fn find_band_period(band_matrix: &Matrix, max_power: usize) -> Option<BandPeriod> {
    // powers[i] は B^(i+1)
    let mut powers = vec![band_matrix.clone()];
    for power in 2..=max_power {
        let next = compose(powers.last().unwrap(), band_matrix);
        for (i, earlier) in powers.iter().enumerate() {
            if let Some(increase) = constant_difference(earlier, &next) {
                return Some(BandPeriod {
                    start_power: i + 1,
                    period: power - (i + 1),
                    increase,
                });
            }
        }
        powers.push(next);
    }
    None
}

/// later が earlier の全成分に同じ値を足したものなら、その値を返す。INF の位置も同じであること
fn constant_difference(earlier: &Matrix, later: &Matrix) -> Option<i64> {
    let mut difference = None;
    for (earlier_row, later_row) in earlier.iter().zip(later.iter()) {
        for (&earlier_loss, &later_loss) in earlier_row.iter().zip(later_row.iter()) {
            match (earlier_loss >= INF, later_loss >= INF) {
                // どちらもつながらない
                (true, true) => continue,
                (false, false) => {
                    let increase = later_loss - earlier_loss;
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
    use crate::matrix::Vector;

    fn matrix(rows: Vec<Vec<i64>>) -> Matrix {
        Matrix(rows.into_iter().map(Vector).collect())
    }

    #[test]
    fn finds_period_one() {
        // B^2 = B + 2
        let band = matrix(vec![vec![2]]);
        assert_eq!(
            find_band_period(&band, 10),
            Some(BandPeriod {
                start_power: 1,
                period: 1,
                increase: 2
            })
        );
    }

    #[test]
    fn finds_period_two_with_inf() {
        // 2 つの状態を行き来するだけ。B^2 は対角だけ有限なので B と INF の位置が違い、B^3 = B + 2
        let band = matrix(vec![vec![INF, 1], vec![1, INF]]);
        assert_eq!(
            find_band_period(&band, 10),
            Some(BandPeriod {
                start_power: 1,
                period: 2,
                increase: 2
            })
        );
    }

    #[test]
    fn no_period_when_growth_differs() {
        // B^r の対角は (r, 2r) で、どの 2 つの累乗の差も成分ごとに違う
        let band = matrix(vec![vec![1, INF], vec![INF, 2]]);
        assert_eq!(find_band_period(&band, 10), None);
    }
}
