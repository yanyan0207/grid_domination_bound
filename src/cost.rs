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
        if top_is_edge && !is_valid_row(row, left_is_edge, right_is_edge, top_is_edge) {
            return INF;
        }

        match self {
            Cost::Stones => stone_count(row),
            Cost::Loss => first_row_loss(row, left_is_edge, right_is_edge, top_is_edge),
        }
    }

    /// 最後の行に行 row を置いたときの追加コスト
    pub fn last_row_addition(
        self,
        _unused_left_is_edge: bool,
        _unused_right_is_edge: bool,
        row: &[Cell],
    ) -> i64 {
        if row.contains(&Cell::NeedOne) {
            return INF;
        }
        match self {
            Cost::Stones => 0,
            Cost::Loss => stone_count(row),
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
fn stone_count(row: &[Cell]) -> i64 {
    row.iter().filter(|&&cell| cell == Cell::Stone).count() as i64
}

/// 1 行目に行 row を置いたときの loss
fn first_row_loss(row: &[Cell], left_is_edge: bool, right_is_edge: bool, top_is_edge: bool) -> i64 {
    let mut cost = 0;

    // 石がエッジにあれば+1
    if left_is_edge && row[0] == Cell::Stone {
        cost += 1;
    }
    if right_is_edge && row[row.len() - 1] == Cell::Stone {
        cost += 1;
    }

    // 石が隣り合う場合は+2
    row.windows(2).for_each(|w| {
        if w[0] == Cell::Stone && w[1] == Cell::Stone {
            cost += 2;
        }
    });

    // 上端に石があれば+1
    if top_is_edge {
        cost += stone_count(row)
    }
    cost
}

/// 前の行 prev の次に行 next を置いたときに増える loss
fn transition_loss(prev: &[Cell], next: &[Cell], left_is_edge: bool, right_is_edge: bool) -> i64 {
    let mut cost = 0;

    // 石がエッジにあれば+1
    if left_is_edge && next[0] == Cell::Stone {
        cost += 1;
    }
    if right_is_edge && next[next.len() - 1] == Cell::Stone {
        cost += 1;
    }

    // 前後の行で石が隣り合う場合は+2
    prev.iter().zip(next.iter()).for_each(|(&p, &n)| {
        if p == Cell::Stone && n == Cell::Stone {
            cost += 2;
        }
    });
    // 次行の石が隣り合う場合は+2
    next.windows(2).for_each(|w| {
        if w[0] == Cell::Stone && w[1] == Cell::Stone {
            cost += 2;
        }
    });
    // 支配値が2を超えてた場合は+1
    // 前後の行で石が隣り合う場合は+2
    prev.iter().zip(next.iter()).for_each(|(&p, &n)| {
        if p == Cell::Ok && n == Cell::Stone {
            cost += 1;
        }
    });
    for i in 1..next.len() - 1 {
        if next[i] == Cell::Ok
            && prev[i] == Cell::Stone
            && next[i - 1] == Cell::Stone
            && next[i + 1] == Cell::Stone
        {
            cost += 1;
        }
    }

    cost
}
