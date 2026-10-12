//! コストの決め方

use crate::{
    rules::{Cell, is_valid_row},
    transfer::INF,
};

/// コストの種類
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cost {
    /// 石の数
    Stones,
    /// loss
    Loss,
}

impl Cost {
    /// 1 行目に行 row を置いたときのコスト
    ///
    /// top_is_edge、行の左端が盤の端かどうか
    pub fn first_row(
        self,
        row: &[Cell],
        left_is_edge: bool,
        right_is_edge: bool,
        top_is_edge: bool,
    ) -> i64 {
        match self {
            Cost::Stones => {
                if top_is_edge && !is_valid_row(row, left_is_edge, right_is_edge, top_is_edge) {
                    INF
                } else {
                    stone_count(row)
                }
            }
            Cost::Loss => first_row_loss(row, left_is_edge, right_is_edge, top_is_edge),
        }
    }

    /// 最後の行に行 row を置いたときの追加コスト
    pub fn last_row_addition(self, left_is_edge: bool, right_is_edge: bool, row: &[Cell]) -> i64 {
        match self {
            Cost::Stones => {
                if row.contains(&Cell::NeedOne) {
                    INF
                } else {
                    0
                }
            }
            Cost::Loss => last_row_loss(row, left_is_edge, right_is_edge),
        }
    }

    /// 前の行 prev の次に行 next を置いたときに増えるコスト
    ///
    /// top_is_edge / bottom_is_edge は、行の左端 / 右端が盤の端かどうか
    pub fn transition(
        self,
        prev: &[Cell],
        next: &[Cell],
        left_is_edge: bool,
        right_is_edge: bool,
    ) -> i64 {
        match self {
            // 石の数は前の行も端も使わない
            Cost::Stones => stone_count(next),
            Cost::Loss => transition_loss(prev, next, left_is_edge, right_is_edge),
        }
    }
}

/// 行の石の数
pub fn stone_count(row: &[Cell]) -> i64 {
    row.iter().filter(|&&cell| cell == Cell::Stone).count() as i64
}

/// 1 行目に行 row を置いたときの loss（未実装）
#[allow(unused)]
pub fn first_row_loss(
    row: &[Cell],
    left_is_edge: bool,
    right_is_edge: bool,
    top_is_edge: bool,
) -> i64 {
    todo!("未実装")
}

/// 最後の行に行 row を置いたときの loss（未実装）
#[allow(unused)]
pub fn last_row_loss(row: &[Cell], left_is_edge: bool, right_is_edge: bool) -> i64 {
    todo!("未実装")
}

/// 前の行 prev の次に行 next を置いたときに増える loss（未実装）
#[allow(unused)]
pub fn transition_loss(
    prev: &[Cell],
    next: &[Cell],
    left_is_edge: bool,
    right_is_edge: bool,
) -> i64 {
    todo!("未実装")
}

/// 盤面全体の loss。ℓ = 6|D| − 2nm（未実装）
#[allow(unused)]
pub fn board_loss(grid: &[&[Cell]]) -> i64 {
    todo!("未実装")
}
