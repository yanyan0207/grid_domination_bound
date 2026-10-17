//! コストの決め方

use crate::{
    rules::{Cell, Edges, Problem, RowEdges, is_valid_row},
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

/// 最小コストから石の数を求める。loss のときは |D| = (2nm + ℓ) / 6
pub fn stone_number(cost: Cost, min_cost: i64, rows: usize, cols: usize) -> i64 {
    if cost == Cost::Loss {
        ((2 * rows * cols) as i64 + min_cost) / 6
    } else {
        min_cost
    }
}

fn _test_is_valid_row() {
    let problem = Problem::Fair2;
    let _edges = Edges {
        top: true,
        bottom: true,
        left: true,
        right: true,
    };

    let cell_list = [Cell::Ok, Cell::Stone, Cell::NeedOne];
    let rows = 3;
    let cols = 3;
    let mut board = vec![vec![Cell::Ok; cols]; rows];
    for i in 0..3usize.pow(9) {
        board[i / cols][i % cols] = cell_list[i % 3];

        // boardば適切か確認
        let mut costs = vec![];
        for row in 0..rows {
            let mut cost = 0;
            for col in 0..cols {
                // 下のセルが不整合の場合の処理
                if row < rows - 1 {
                    if problem == Problem::Fair2
                        && board[row][col] == Cell::Ok
                        && board[row + 1][col] == Cell::Stone
                    {
                        cost = INF;
                        break;
                    }
                    if board[row][col] == Cell::NeedOne && board[row + 1][col] != Cell::Stone {
                        cost = INF;
                        break;
                    }
                }
                // 周りの石の数を数える
                let mut stone_except_below = 0;
                if row > 0 && board[row - 1][col] == Cell::Stone {
                    stone_except_below += 1;
                }
                if col > 0 && board[row][col - 1] == Cell::Stone {
                    stone_except_below += 1;
                }
                if col < cols - 1 && board[row][col + 1] == Cell::Stone {
                    stone_except_below += 1;
                }
                let mut stone_arround = stone_except_below;
                if row < rows - 1 && board[row + 1][col] == Cell::Stone {
                    stone_arround += 1;
                }

                match board[row][col] {
                    Cell::Ok => {
                        if stone_except_below < 2 {
                            cost = INF;
                            break;
                        } else if stone_except_below > 2 {
                            if problem == Problem::Fair2 {
                                cost = INF;
                                break;
                            } else {
                                cost += stone_arround - 2;
                            }
                        }
                    }
                    Cell::NeedOne => {
                        if stone_except_below != 1 {
                            cost = INF;
                        }
                    }
                    Cell::Stone => {
                        cost += stone_arround;
                        if row == 0 || row == rows - 1 {
                            cost += 1;
                        }
                        if col == 0 || col == cols - 1 {
                            cost += 1;
                        }
                    }
                }
            }
            costs.push(cost);
        }
        for i in 0..rows {
            assert_eq!(
                (costs[i] != INF),
                is_valid_row(
                    &board[i],
                    &Edges {
                        top: i == 0,
                        bottom: i == rows - 1,
                        left: true,
                        right: true
                    }
                )
            );
        }
    }
}
