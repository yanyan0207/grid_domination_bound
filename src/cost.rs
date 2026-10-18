//! コストの決め方

use crate::{
    rules::{Cell, Edges, RowEdges, is_valid_row},
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
    /// edges は盤の四方が端かどうか
    pub fn first_row(self, row: &[Cell], edges: &Edges) -> i64 {
        if edges.top && !is_valid_row(row, edges) {
            return INF;
        }

        match self {
            Cost::Stones => stone_count(row),
            Cost::Loss => first_row_loss(row, edges),
        }
    }

    /// 最後の行に行 row を置いたときの追加コスト
    pub fn last_row_addition(self, _unused_row_edges: RowEdges, row: &[Cell]) -> i64 {
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
    /// row_edges は、行の左端 / 右端が盤の端かどうか
    pub fn transition(self, prev: &[Cell], next: &[Cell], row_edges: RowEdges) -> i64 {
        match self {
            // 石の数は前の行も端も使わない
            Cost::Stones => stone_count(next),
            Cost::Loss => transition_loss(prev, next, row_edges),
        }
    }
}

/// 行の石の数
fn stone_count(row: &[Cell]) -> i64 {
    row.iter().filter(|&&cell| cell == Cell::Stone).count() as i64
}

/// 1 行目に行 row を置いたときの loss
fn first_row_loss(row: &[Cell], edges: &Edges) -> i64 {
    let Edges {
        top: top_is_edge,
        bottom: _bottom_is_edge,
        left: left_is_edge,
        right: right_is_edge,
    } = *edges;
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
fn transition_loss(prev: &[Cell], next: &[Cell], row_edges: RowEdges) -> i64 {
    let RowEdges {
        left: left_is_edge,
        right: right_is_edge,
    } = row_edges;
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

/// 最小コストから石の数を求める。loss のときは 6|D| = 2nm + ℓ から |D| = (2nm + ℓ) / 6
///
/// 実際の置き方なら 2nm + ℓ は 6 で割り切れる。縁の計算のように ℓ が下界のときは割り切れないことがあり、
/// 石の数は整数なので、6 で割って切り上げた値が石の数の下界になる
pub fn stone_number(cost: Cost, min_cost: i64, rows: usize, cols: usize) -> i64 {
    if cost == Cost::Loss {
        let six_times_stones = (2 * rows * cols) as i64 + min_cost;
        // 6 で割って切り上げる
        (six_times_stones + 5) / 6
    } else {
        min_cost
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::{Edges, list_valid_rows};

    use super::*;

    #[test_case::test_matrix(
    [2,3,4,5],[true,false],[true,false]
)]
    fn test_first_row_is_invalid(width: usize, left: bool, right: bool) {
        let edges = Edges {
            top: true,
            bottom: true,
            left,
            right,
        };

        let valid_rows = list_valid_rows(width, edges.row());

        for row in valid_rows {
            let actual = Cost::Loss.first_row(&row, &edges) != INF;
            let mut expected = true;
            for (i, cell) in row.iter().enumerate() {
                if cell == &Cell::Ok {
                    if (left && i == 0)
                        || (right && i == width - 1)
                        || (i > 0 && row[i - 1] != Cell::Stone)
                        || (i < width - 1 && row[i + 1] != Cell::Stone)
                    {
                        expected = false;
                        break;
                    }
                } else if cell == &Cell::NeedOne
                    && ((i == 0 && left) || (i > 0 && row[i - 1] != Cell::Stone))
                    && ((i == width - 1 && right) || (i < width - 1 && row[i + 1] != Cell::Stone))
                {
                    expected = false;
                    break;
                }
            }
            assert_eq!(
                actual, expected,
                "Failed for row: {:?} expected: {} actual: {}",
                row, expected, actual
            );
        }
    }
}
