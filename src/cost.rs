//! コストの決め方

use crate::{
    rules::{
        Cell, ColumnPlace, Edges, Graph, Problem, RowEdges, SAME_ROW_COLUMN_OFFSETS, column_place,
        count_existing_positions, count_stones_and_unknown, is_valid_row,
    },
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
    pub fn first_row(self, graph: Graph, problem: Problem, row: &[Cell], edges: &Edges) -> i64 {
        if edges.top && !is_valid_row(row, graph, problem, edges) {
            return INF;
        }

        match self {
            Cost::Stones => stone_count(row),
            Cost::Loss => first_row_loss(graph, row, edges),
        }
    }

    /// 最後の行に行 row を置いたときの追加コスト
    pub fn last_row_addition(self, graph: Graph, row_edges: RowEdges, row: &[Cell]) -> i64 {
        // 次の行から石が必要なセルが残っていたら、最後の行にはできない
        if row
            .iter()
            .any(|&cell| cell != Cell::Stone && cell.needed_from_next_row() > 0)
        {
            return INF;
        }
        match self {
            Cost::Stones => 0,
            Cost::Loss => last_row_loss(graph, row, row_edges),
        }
    }

    /// 前の行 prev の次に行 next を置いたときに増えるコスト
    ///
    /// row_edges は、行の左端 / 右端が盤の端かどうか
    pub fn transition(self, graph: Graph, prev: &[Cell], next: &[Cell], row_edges: RowEdges) -> i64 {
        match self {
            // 石の数は前の行も端も使わない
            Cost::Stones => stone_count(next),
            Cost::Loss => transition_loss(graph, prev, next, row_edges),
        }
    }
}

/// loss の式 (次数 + 2)|D| = 2nm + ℓ の、石の数 |D| の係数（Grid は 6、King は 10）
///
/// 石でないマスはそれぞれ石に 2 回（過剰な分は ℓ に入れる）隣接し、石は次数の分だけ隣接できるが、
/// 盤の端で欠けた隣と、石どうしの隣接（2 回ずつ）の分が ℓ になる
pub fn stones_coefficient(graph: Graph) -> i64 {
    (graph.degree() + 2) as i64
}

/// 行の石の数
fn stone_count(row: &[Cell]) -> i64 {
    row.iter().filter(|&&cell| cell == Cell::Stone).count() as i64
}

/// 列 col の石が、盤の端の外で失う隣のうち、列が端の外にある分の数
///
/// 同じ行の左右と、前後の行の隣列を数える（King は片側 3 つ、Grid は 1 つ）
fn lost_neighbors_outside_columns(
    graph: Graph,
    col: usize,
    width: usize,
    row_edges: RowEdges,
) -> i64 {
    let is_outside_edge =
        |&&offset: &&isize| column_place(col, offset, width, row_edges) == ColumnPlace::OutsideEdge;
    let same_row = SAME_ROW_COLUMN_OFFSETS.iter().filter(is_outside_edge).count();
    let each_adjacent_row = graph
        .adjacent_row_column_offsets()
        .iter()
        .filter(is_outside_edge)
        .count();
    (same_row + 2 * each_adjacent_row) as i64
}

/// 列 col の石が、前の行（または次の行）が盤の端の外でないことで失う隣の数
///
/// 列が端の外にある分は lost_neighbors_outside_columns で数えるので、ここでは除く
fn lost_neighbors_in_missing_row(
    graph: Graph,
    col: usize,
    width: usize,
    row_edges: RowEdges,
) -> i64 {
    count_existing_positions(col, graph.adjacent_row_column_offsets(), width, row_edges) as i64
}

/// 同じ行で石が隣り合う組ごとに 2
fn same_row_stone_pairs_loss(row: &[Cell]) -> i64 {
    row.windows(2)
        .filter(|pair| pair[0] == Cell::Stone && pair[1] == Cell::Stone)
        .count() as i64
        * 2
}

/// 1 行目に行 row を置いたときの loss
fn first_row_loss(graph: Graph, row: &[Cell], edges: &Edges) -> i64 {
    let width = row.len();
    let row_edges = edges.row();
    let mut loss = 0;
    for (col, &cell) in row.iter().enumerate() {
        if cell != Cell::Stone {
            continue;
        }
        loss += lost_neighbors_outside_columns(graph, col, width, row_edges);
        // 上端なら、前の行がないことで失う隣
        if edges.top {
            loss += lost_neighbors_in_missing_row(graph, col, width, row_edges);
        }
    }
    loss + same_row_stone_pairs_loss(row)
}

/// 前の行 prev の次に行 next を置いたときに増える loss
fn transition_loss(graph: Graph, prev: &[Cell], next: &[Cell], row_edges: RowEdges) -> i64 {
    let width = next.len();
    let adjacent_offsets = graph.adjacent_row_column_offsets();
    let mut loss = 0;

    for (col, &cell) in next.iter().enumerate() {
        if cell == Cell::Stone {
            loss += lost_neighbors_outside_columns(graph, col, width, row_edges);
            // 前の行の石と隣り合う組ごとに 2
            let in_prev_row = count_stones_and_unknown(prev, col, adjacent_offsets, row_edges);
            loss += 2 * in_prev_row.stones as i64;
        } else {
            // 周り（前の行と同じ行）の石が 2 を超えた分（2-domination の過剰な支配）
            let around = count_stones_and_unknown(prev, col, adjacent_offsets, row_edges)
                + count_stones_and_unknown(next, col, &SAME_ROW_COLUMN_OFFSETS, row_edges);
            loss += around.stones.saturating_sub(2) as i64;
        }
    }
    loss += same_row_stone_pairs_loss(next);

    // 前の行の石でないセルが、次の行から必要な数を超えて石をもらった分（2-domination の過剰な支配）
    for (col, &cell) in prev.iter().enumerate() {
        if cell == Cell::Stone {
            continue;
        }
        let in_next_row = count_stones_and_unknown(next, col, adjacent_offsets, row_edges);
        loss += in_next_row
            .stones
            .saturating_sub(cell.needed_from_next_row()) as i64;
    }
    loss
}

/// 最後の行（下端）に行 row を置いたときに、次の行がないことで石が失う隣の数
fn last_row_loss(graph: Graph, row: &[Cell], row_edges: RowEdges) -> i64 {
    let width = row.len();
    row.iter()
        .enumerate()
        .filter(|&(_, &cell)| cell == Cell::Stone)
        .map(|(col, _)| lost_neighbors_in_missing_row(graph, col, width, row_edges))
        .sum()
}

/// 最小コストから石の数を求める。loss のときは (次数 + 2)|D| = 2nm + ℓ から |D| = (2nm + ℓ) / (次数 + 2)
///
/// 実際の置き方なら 2nm + ℓ は次数 + 2 で割り切れる。縁の計算のように ℓ が下界のときは割り切れないことがあり、
/// 石の数は整数なので、割って切り上げた値が石の数の下界になる
pub fn stone_number(graph: Graph, cost: Cost, min_cost: i64, rows: usize, cols: usize) -> i64 {
    if cost == Cost::Loss {
        let coefficient = stones_coefficient(graph);
        let coefficient_times_stones = (2 * rows * cols) as i64 + min_cost;
        // 切り上げる
        (coefficient_times_stones + coefficient - 1) / coefficient
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

        let valid_rows = list_valid_rows(width, Graph::Grid, Problem::Fair2, edges.row());

        for row in valid_rows {
            let actual = Cost::Loss.first_row(Graph::Grid, Problem::Fair2, &row, &edges) != INF;
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

    /// King で盤の端の石が失う隣の数。辺は 3、角は 5
    #[test]
    fn king_lost_neighbors_at_edges() {
        let row_edges = RowEdges {
            left: true,
            right: true,
        };
        // 幅 3 の 1 行目（上端）に石を 1 つ置く
        let top_edges = Edges {
            top: true,
            bottom: false,
            left: true,
            right: true,
        };
        let corner = vec![Cell::Stone, Cell::NeedTwo, Cell::NeedTwo];
        assert_eq!(first_row_loss(Graph::King, &corner, &top_edges), 5);
        let middle = vec![Cell::NeedTwo, Cell::Stone, Cell::NeedTwo];
        assert_eq!(first_row_loss(Graph::King, &middle, &top_edges), 3);
        // 途中の行の左端の石は 3。前の行に石はない
        let prev = vec![Cell::NeedTwo, Cell::NeedTwo, Cell::NeedTwo];
        assert_eq!(transition_loss(Graph::King, &prev, &corner, row_edges), 3);
        // 最後の行（下端）の左端の石は、次の行がないことで 2 を失う
        assert_eq!(last_row_loss(Graph::King, &corner, row_edges), 2);
    }

    /// King では斜めに並んだ石も隣り合う組として 2 を足す
    #[test]
    fn king_diagonal_stone_pair() {
        let row_edges = RowEdges {
            left: false,
            right: false,
        };
        let prev = vec![Cell::Stone, Cell::NeedOne, Cell::NeedTwo];
        let next = vec![Cell::NeedOne, Cell::Stone, Cell::NeedOne];
        assert_eq!(transition_loss(Graph::King, &prev, &next, row_edges), 2);
        assert_eq!(transition_loss(Graph::Grid, &prev, &next, row_edges), 0);
    }
}
