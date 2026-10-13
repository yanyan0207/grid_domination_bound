//! 縁（盤の端から幅 corner_size の部分。四隅の角と 4 本の帯）の最小 loss の計算
//!
//! 角（corner）と帯（band）の計算を組み合わせて、盤を一周する

use crate::{cost::Cost, rules::Problem};

/// 幅 cols、長さ rows の帯の最小コストの計算（未実装）。corner_size は角の一辺
pub fn border_cost_matrix(
    rows: usize,
    cols: usize,
    corner_size: usize,
    problem: Problem,
    cost: Cost,
    verbose: bool,
) {
    println!(
        "band: rows = {rows}, cols = {cols}, corner_size = {corner_size}, problem = {problem}, cost = {cost:?}, verbose = {verbose}"
    );
}
