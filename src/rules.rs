//! 問題ごとの規則（列を並べられるかの判定）

use std::ops::Add;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Problem {
    /// 2-domination
    Dom2,
    /// 2-fair
    Fair2,
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Problem::Dom2 => "2-domination",
            Problem::Fair2 => "2-fair",
        })
    }
}

/// 盤のグラフの種類。どのマスどうしが隣になるか
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Graph {
    /// グリッドグラフ。縦横の 4 方向が隣
    Grid,
    /// キンググラフ。縦横斜めの 8 方向が隣（チェスのキングが 1 手で動ける先）
    King,
}

impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Graph::Grid => "grid",
            Graph::King => "king",
        })
    }
}

/// 同じ行で隣になる列のずれ（左隣と右隣）。グラフの種類によらない
pub const SAME_ROW_COLUMN_OFFSETS: [isize; 2] = [-1, 1];

impl Graph {
    /// 前の行や次の行で、列 col の隣になる列のずれ
    pub fn adjacent_row_column_offsets(self) -> &'static [isize] {
        match self {
            Graph::Grid => &[0],
            Graph::King => &[-1, 0, 1],
        }
    }

    /// 盤の内側のマスの隣の数（Grid は 4、King は 8）
    pub fn degree(self) -> usize {
        SAME_ROW_COLUMN_OFFSETS.len() + 2 * self.adjacent_row_column_offsets().len()
    }

    /// 行のセルに使う状態。Grid では次の行に隣が 1 つしかないので NeedTwo は使わない
    pub fn cell_states(self) -> &'static [Cell] {
        match self {
            Graph::Grid => &[Cell::Stone, Cell::Ok, Cell::NeedOne],
            Graph::King => &[Cell::Stone, Cell::Ok, Cell::NeedOne, Cell::NeedTwo],
        }
    }
}

/// 盤の四方が盤の端かどうか
///
/// 行を上から順に並べるとき、top は 1 行目の外側、bottom は最後の行の外側、
/// left / right は各行の左端 / 右端の外側
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edges {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

/// 行の左端 / 右端が盤の端かどうか
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowEdges {
    pub left: bool,
    pub right: bool,
}

impl Edges {
    /// 行の左端 / 右端だけを取り出す
    pub fn row(self) -> RowEdges {
        RowEdges {
            left: self.left,
            right: self.right,
        }
    }

    /// 四方とも盤の端（盤全体）
    pub const ALL: Edges = Edges {
        top: true,
        bottom: true,
        left: true,
        right: true,
    };
}

/// セルの状態。前の行と同じ行の石を数えた時点で、次の行に何が必要かを表す
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    /// 石
    Stone,
    /// 周りに必要な石がある
    Ok,
    /// 次の行から石が 1 つ必要
    NeedOne,
    /// 次の行から石が 2 つ必要（King だけ。次の行に隣が 3 つあるので起こりうる）
    NeedTwo,
}

impl Cell {
    /// 石でないセルが、次の行から必要とする石の数
    pub fn needed_from_next_row(self) -> usize {
        match self {
            Cell::Ok => 0,
            Cell::NeedOne => 1,
            Cell::NeedTwo => 2,
            Cell::Stone => unreachable!("石には次の行から必要な石の数はない"),
        }
    }
}

/// 列 col からずれ offset の位置が、行の中か、行の外か
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColumnPlace {
    /// 行の中。その列
    Inside(usize),
    /// 行の外で、盤の内側（マスはあるが、何があるかまだ分からない）
    OutsideOpen,
    /// 行の外で、盤の端の外（マスがない）
    OutsideEdge,
}

/// 幅 width の行で、列 col からずれ offset の位置がどこになるか
pub fn column_place(col: usize, offset: isize, width: usize, row_edges: RowEdges) -> ColumnPlace {
    let shifted = col as isize + offset;
    if shifted < 0 {
        if row_edges.left {
            ColumnPlace::OutsideEdge
        } else {
            ColumnPlace::OutsideOpen
        }
    } else if shifted >= width as isize {
        if row_edges.right {
            ColumnPlace::OutsideEdge
        } else {
            ColumnPlace::OutsideOpen
        }
    } else {
        ColumnPlace::Inside(shifted as usize)
    }
}

/// 隣の位置を見た結果。分かっている石の数と、まだ分からない位置の数
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StonesAndUnknown {
    /// 石の数
    pub stones: usize,
    /// 行の外で盤の内側にあって、何があるかまだ分からない位置の数
    pub unknown: usize,
}

impl Add for StonesAndUnknown {
    type Output = StonesAndUnknown;

    fn add(self, other: StonesAndUnknown) -> StonesAndUnknown {
        StonesAndUnknown {
            stones: self.stones + other.stones,
            unknown: self.unknown + other.unknown,
        }
    }
}

/// 行 row で、列 col からずれ offsets の位置を見て、石の数と、まだ分からない位置の数を数える
pub fn count_stones_and_unknown(
    row: &[Cell],
    col: usize,
    offsets: &[isize],
    row_edges: RowEdges,
) -> StonesAndUnknown {
    let mut counted = StonesAndUnknown::default();
    for &offset in offsets {
        match column_place(col, offset, row.len(), row_edges) {
            ColumnPlace::Inside(neighbor_col) => {
                if row[neighbor_col] == Cell::Stone {
                    counted.stones += 1;
                }
            }
            ColumnPlace::OutsideOpen => counted.unknown += 1,
            ColumnPlace::OutsideEdge => {}
        }
    }
    counted
}

/// 列 col からずれ offsets の位置のうち、マスがある（行の中か、行の外で盤の内側）ものの数
pub fn count_existing_positions(
    col: usize,
    offsets: &[isize],
    width: usize,
    row_edges: RowEdges,
) -> usize {
    offsets
        .iter()
        .filter(|&&offset| column_place(col, offset, width, row_edges) != ColumnPlace::OutsideEdge)
        .count()
}

/// 石でないセルの状態 cell が、周り（前の行と同じ行）の石の数に合うか
///
/// 周りの石の数は around.stones 以上 around.stones + around.unknown 以下のどれか
/// （分からない位置は石でも空でもよい。縁の計算ではこれが下界になる）。
/// 2-fair では、周りの石と次の行から必要な石を合わせてちょうど 2。
/// 2-domination では、Ok（次の行から必要なし）は周りの石が 2 以上ならよく、それ以外は 2-fair と同じ
pub fn cell_state_fits(problem: Problem, cell: Cell, around: StonesAndUnknown) -> bool {
    let needed = cell.needed_from_next_row();
    let stones_around_without_next_row = 2 - needed;
    let least = around.stones;
    let most = around.stones + around.unknown;
    let fits_exactly = least <= stones_around_without_next_row && stones_around_without_next_row <= most;
    match problem {
        Problem::Fair2 => fits_exactly,
        Problem::Dom2 => {
            if needed == 0 {
                most >= 2
            } else {
                fits_exactly
            }
        }
    }
}

/// 石でないセルが次の行から必要とする石の数 needed に、次の行の隣の位置 in_next_row が合うか
pub fn next_row_fits_need(problem: Problem, needed: usize, in_next_row: StonesAndUnknown) -> bool {
    match problem {
        Problem::Fair2 => {
            in_next_row.stones <= needed && needed <= in_next_row.stones + in_next_row.unknown
        }
        Problem::Dom2 => in_next_row.stones + in_next_row.unknown >= needed,
    }
}

/// 指定された幅のすべての可能な行のリストから、問題に従って有効な行だけを返す
pub fn list_valid_rows(
    width: usize,
    graph: Graph,
    problem: Problem,
    row_edges: RowEdges,
) -> Vec<Vec<Cell>> {
    let cell_states = graph.cell_states();
    let state_count = cell_states.len() as u64;
    let mut valid_rows: Vec<Vec<Cell>> = Vec::new();
    for row_number in 0..state_count.pow(width as u32) {
        let mut row = Vec::with_capacity(width);
        let mut rest = row_number;
        for _ in 0..width {
            row.push(cell_states[(rest % state_count) as usize]);
            rest /= state_count;
        }
        // 途中の行として有効か（上下は端でない）
        let edges = Edges {
            top: false,
            bottom: false,
            left: row_edges.left,
            right: row_edges.right,
        };
        if is_valid_row(&row, graph, problem, &edges) {
            valid_rows.push(row);
        }
    }
    valid_rows
}

/// 遷移が有効かどうかを判定
///
/// 遷移の一覧は list_next_rows で作るので、本体では使わない。テストで、作った遷移の検算に使う
#[allow(dead_code)]
pub fn is_valid_transition(
    prev_row: &[Cell],
    next_row: &[Cell],
    graph: Graph,
    problem: Problem,
    row_edges: RowEdges,
) -> bool {
    (0..next_row.len()).all(|col| {
        prev_cell_need_fits(prev_row, next_row, col, graph, problem, row_edges)
            && next_cell_state_fits(prev_row, next_row, col, graph, problem, row_edges)
    })
}

/// 前の行の列 col のセルが次の行に求める石が、次の行の隣の位置にあるか（石なら何も求めない）
///
/// 次の行は、列 col の隣の位置（col ± 前後の行のずれ）まで決まっていればよい
fn prev_cell_need_fits(
    prev_row: &[Cell],
    next_row: &[Cell],
    col: usize,
    graph: Graph,
    problem: Problem,
    row_edges: RowEdges,
) -> bool {
    let prev_cell = prev_row[col];
    if prev_cell == Cell::Stone {
        return true;
    }
    let in_next_row = count_stones_and_unknown(
        next_row,
        col,
        graph.adjacent_row_column_offsets(),
        row_edges,
    );
    next_row_fits_need(problem, prev_cell.needed_from_next_row(), in_next_row)
}

/// 次の行の列 col のセルの状態が、前の行と同じ行の石に合うか（石なら常に合う）
///
/// 次の行は、列 col の両隣まで決まっていればよい
fn next_cell_state_fits(
    prev_row: &[Cell],
    next_row: &[Cell],
    col: usize,
    graph: Graph,
    problem: Problem,
    row_edges: RowEdges,
) -> bool {
    let next_cell = next_row[col];
    if next_cell == Cell::Stone {
        return true;
    }
    let adjacent_offsets = graph.adjacent_row_column_offsets();
    let around = count_stones_and_unknown(prev_row, col, adjacent_offsets, row_edges)
        + count_stones_and_unknown(next_row, col, &SAME_ROW_COLUMN_OFFSETS, row_edges);
    if !cell_state_fits(problem, next_cell, around) {
        return false;
    }
    // 次の行から必要な石の数が、次の行で隣になる位置の数を超えない
    let next_row_positions =
        count_existing_positions(col, adjacent_offsets, next_row.len(), row_edges);
    next_cell.needed_from_next_row() <= next_row_positions
}

/// 前の行 prev_row から遷移できる次の行を全部作る
///
/// 列を左から順に決めていき、決まった列まででできる判定をその場で行って、合わない枝は先に進まない。
/// 有効な行どうしの全部の組を調べるより、遷移が少ないときにずっと速い
pub fn list_next_rows(
    prev_row: &[Cell],
    graph: Graph,
    problem: Problem,
    row_edges: RowEdges,
) -> Vec<Vec<Cell>> {
    let width = prev_row.len();
    // 前の行の列 col の判定には、次の行が列 col + farthest_offset まで決まっている必要がある
    let farthest_offset = *graph
        .adjacent_row_column_offsets()
        .iter()
        .max()
        .expect("前後の行で隣になる列がある") as usize;
    // まだ決めていない列の仮の値。判定は決まった列しか読まない
    let mut next_row = vec![Cell::Ok; width];
    let mut next_rows = vec![];
    fill_next_row_from_column(
        0,
        prev_row,
        &mut next_row,
        graph,
        problem,
        row_edges,
        farthest_offset,
        &mut next_rows,
    );
    next_rows
}

/// list_next_rows の本体。next_row の列 col より前は決まっていて、列 col から先を決める
#[allow(clippy::too_many_arguments)]
fn fill_next_row_from_column(
    col: usize,
    prev_row: &[Cell],
    next_row: &mut Vec<Cell>,
    graph: Graph,
    problem: Problem,
    row_edges: RowEdges,
    farthest_offset: usize,
    next_rows: &mut Vec<Vec<Cell>>,
) {
    let width = prev_row.len();
    if col == width {
        // 全部決まったら、残りの判定（前の行の最後の farthest_offset 列と、次の行の最後の列）
        let prev_cols_left = (width - farthest_offset)..width;
        let prev_fits = prev_cols_left.into_iter().all(|prev_col| {
            prev_cell_need_fits(prev_row, next_row, prev_col, graph, problem, row_edges)
        });
        if prev_fits
            && next_cell_state_fits(prev_row, next_row, width - 1, graph, problem, row_edges)
        {
            next_rows.push(next_row.clone());
        }
        return;
    }
    for &cell in graph.cell_states() {
        next_row[col] = cell;
        // 列 col まで決まると、前の行の列 col − farthest_offset と、次の行の列 col − 1 の判定ができる
        let prev_fits = col < farthest_offset
            || prev_cell_need_fits(
                prev_row,
                next_row,
                col - farthest_offset,
                graph,
                problem,
                row_edges,
            );
        let next_fits = col == 0
            || next_cell_state_fits(prev_row, next_row, col - 1, graph, problem, row_edges);
        if prev_fits && next_fits {
            fill_next_row_from_column(
                col + 1,
                prev_row,
                next_row,
                graph,
                problem,
                row_edges,
                farthest_offset,
                next_rows,
            );
        }
    }
}

/// 指定された行が問題に従って有効かどうかを判定
///
/// 同じ行の石だけを見る。上が開いていれば、前の行の隣の位置は全部まだ分からないとする
pub fn is_valid_row(row: &[Cell], graph: Graph, problem: Problem, edges: &Edges) -> bool {
    let width = row.len();
    let row_edges = edges.row();
    let adjacent_offsets = graph.adjacent_row_column_offsets();
    for (col, &cell) in row.iter().enumerate() {
        if cell == Cell::Stone {
            continue;
        }
        let mut around = count_stones_and_unknown(row, col, &SAME_ROW_COLUMN_OFFSETS, row_edges);
        if !edges.top {
            around.unknown += count_existing_positions(col, adjacent_offsets, width, row_edges);
        }
        if !cell_state_fits(problem, cell, around) {
            return false;
        }
        // 次の行から必要な石の数が、次の行で隣になる位置の数を超えない
        let next_row_positions = count_existing_positions(col, adjacent_offsets, width, row_edges);
        if cell.needed_from_next_row() > next_row_positions {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use super::*;
    const CELL_LIST: [Cell; 3] = [Cell::Ok, Cell::Stone, Cell::NeedOne];

    /// Grid の有効な行のリスト（問題によらないので 2-domination で調べる）
    fn grid_valid_rows(width: usize, left: bool, right: bool) -> Vec<Vec<Cell>> {
        list_valid_rows(width, Graph::Grid, Problem::Dom2, RowEdges { left, right })
    }

    #[test]
    fn test_list_valid_rows_2() {
        let rows = grid_valid_rows(2, true, true);
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok]));
        assert_eq!(rows.len(), 6);
    }

    #[test]
    fn test_list_valid_rows_2_left_not_edge() {
        let rows = grid_valid_rows(2, false, true);
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok]));
        assert_eq!(rows.len(), 7);
    }

    #[test]
    fn test_list_valid_rows_2_right_not_edge() {
        let rows = grid_valid_rows(2, true, false);
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne]));
        assert_eq!(rows.len(), 7);
    }

    #[test]
    fn test_list_valid_rows_2_both_not_edge() {
        let rows = grid_valid_rows(2, false, false);
        assert_eq!(rows.len(), 9);
    }

    #[test]
    fn test_list_valid_rows_3() {
        let rows = grid_valid_rows(3, true, true);
        assert!(!rows.contains(&vec![Cell::Stone, Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Stone, Cell::NeedOne, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Stone, Cell::NeedOne, Cell::Stone]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok, Cell::Stone]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok, Cell::NeedOne]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne, Cell::Stone]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne, Cell::NeedOne]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok, Cell::NeedOne]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::NeedOne, Cell::Ok]));
        assert_eq!(rows.len(), 15);
    }

    #[test]
    fn test_list_valid_rows_3_left_not_edge() {
        let rows = grid_valid_rows(3, false, true);
        assert!(!rows.contains(&vec![Cell::Stone, Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Stone, Cell::NeedOne, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Stone, Cell::NeedOne, Cell::Stone]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok, Cell::NeedOne]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok, Cell::NeedOne]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::NeedOne, Cell::Ok]));
        assert_eq!(rows.len(), 18);
    }

    #[test]
    fn test_list_valid_rows_3_right_not_edge() {
        let rows = grid_valid_rows(3, true, false);
        assert!(!rows.contains(&vec![Cell::Stone, Cell::NeedOne, Cell::Stone]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok, Cell::Stone]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok, Cell::NeedOne]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne, Cell::Stone]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne, Cell::NeedOne]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok, Cell::NeedOne]));
        assert_eq!(rows.len(), 18);
    }

    #[test]
    fn test_list_valid_rows_3_both_not_edge() {
        let rows = grid_valid_rows(3, false, false);
        assert!(!rows.contains(&vec![Cell::Stone, Cell::NeedOne, Cell::Stone]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok, Cell::NeedOne]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok, Cell::NeedOne]));
        assert_eq!(rows.len(), 22);
    }

    #[test_case::test_matrix(
    [true,false],[true,false],[2,3,4,5]
)]
    fn test_list_valid_rows_4(left: bool, right: bool, width: usize) {
        let rows = grid_valid_rows(width, left, right);

        let mut expected_rows = vec![];
        'outer: for i in 0..3usize.pow(width as u32) {
            let mut row = vec![];
            for j in 0..width {
                row.push(CELL_LIST[i / 3usize.pow(j as u32) % 3]);
            }

            // 左辺エッジ OK==0
            if left && row[0] == Cell::Ok && (row[1] != Cell::Stone) {
                continue;
            }
            // 真ん中
            for j in 1..row.len() - 1 {
                //  OK==0
                if row[j - 1] != Cell::Stone && row[j] == Cell::Ok && row[j + 1] != Cell::Stone {
                    continue 'outer;
                }
                // NeedOne == 2
                if row[j - 1] == Cell::Stone && row[j] == Cell::NeedOne && row[j + 1] == Cell::Stone
                {
                    continue 'outer;
                }
            }
            // 右辺エッジ
            if right && row[row.len() - 1] == Cell::Ok && (row[row.len() - 2] != Cell::Stone) {
                continue;
            }
            expected_rows.push(row);
        }
        let actual = HashSet::<&Vec<Cell>>::from_iter(rows.iter());
        let expected = HashSet::<&Vec<Cell>>::from_iter(expected_rows.iter());
        println!("Actual: {:?}", actual);
        println!("Expected: {:?}", expected);
        assert_eq!(
            actual,
            expected,
            "\n過剰: {:?}\n不足: {:?}",
            actual.difference(&expected).collect::<Vec<_>>(),
            expected.difference(&actual).collect::<Vec<_>>(),
        );
    }

    /// King の行の列挙では、Grid の行がそのまま含まれ、NeedTwo の行が増える
    #[test]
    fn king_rows_include_grid_rows() {
        let grid_rows = grid_valid_rows(3, true, true);
        let king_rows = list_valid_rows(
            3,
            Graph::King,
            Problem::Fair2,
            RowEdges {
                left: true,
                right: true,
            },
        );
        for row in &grid_rows {
            assert!(king_rows.contains(row), "Grid の行 {row:?} が King にない");
        }
        // NeedTwo は隣に石がないときだけ
        assert!(king_rows.contains(&vec![Cell::NeedTwo, Cell::NeedTwo, Cell::NeedTwo]));
        assert!(!king_rows.contains(&vec![Cell::Stone, Cell::NeedTwo, Cell::Ok]));
        assert!(king_rows.contains(&vec![Cell::Stone, Cell::NeedOne, Cell::NeedTwo]));
    }

    /// King の遷移: 前の行の NeedTwo の下に、斜めも含めて石がちょうど 2 つ
    #[test]
    fn king_transition_need_two() {
        let row_edges = RowEdges {
            left: true,
            right: true,
        };
        let prev = vec![Cell::NeedOne, Cell::NeedTwo, Cell::NeedOne];
        // 真ん中の NeedTwo の下 3 つのうち両端の 2 つが石。両端の NeedOne は下 2 つのうち 1 つが石。
        // 次の行の真ん中は両隣が石なので Ok
        let next = vec![Cell::Stone, Cell::Ok, Cell::Stone];
        assert!(is_valid_transition(&prev, &next, Graph::King, Problem::Fair2, row_edges));
        // 石が 3 つ並ぶと、NeedTwo の下に 3 つ、NeedOne の下に 2 つで、2-fair では多すぎる
        let next = vec![Cell::Stone, Cell::Stone, Cell::Stone];
        assert!(!is_valid_transition(&prev, &next, Graph::King, Problem::Fair2, row_edges));
        assert!(is_valid_transition(&prev, &next, Graph::King, Problem::Dom2, row_edges));
    }
}
