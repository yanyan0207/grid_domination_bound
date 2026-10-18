//! 角（盤の隅の、一辺 corner_size の正方形）の計算
//!
//! 角の入力列 in_column（角の右隣の列。上の帯の最後の行で、状態は左向き）ごとに、角の行を上から積んで、
//! 出力行 out_row（下の帯につながる行）までの最小コストを求める。
//! Grid は転送行列の DP で計算する。King は、in_column のセルが角の最後の列の 3 行（上・同じ行・下）と隣になり、
//! 1 行前しか持たない DP では判定できないので、1 つ前の行の最後のセルも状態に持つ DP で計算する

use crate::display::{BoardView, Dir};
use crate::dp::{calc_dp, init_dp, print_dp_result};
use crate::matrix::{Matrix, SparseMatrix, Vector};
use crate::rules::{
    Cell, Edges, Graph, RowEdges, SAME_ROW_COLUMN_OFFSETS, StonesAndUnknown, cell_state_fits,
    count_stones_and_unknown, list_valid_rows, next_row_fits_need,
};
use crate::transfer::build_transition_cost_matrix;
use crate::{cost::Cost, rules::Problem, transfer::INF};

/// 一辺 corner_size の角の計算
pub fn corner(corner_size: usize, graph: Graph, problem: Problem, cost: Cost, verbose: bool) {
    println!(
        "corner: corner_size = {corner_size}, graph = {graph}, problem = {problem}, cost = {cost:?}, verbose = {verbose}"
    );

    let CornerMatrixResult {
        valid_rows,
        cost_matrix,
        paths,
        rows_in_corner: _,
    } = corner_transition_matrix(corner_size, graph, problem, cost, verbose);

    // 入力列自身のコストも足して、最小の (入力列, 出力行) を探す
    let in_column_edges = Edges {
        top: false,
        bottom: false,
        left: true,
        right: false,
    };
    let mut min_value = INF;
    let mut min_in_col_index = 0;
    let mut min_out_row_index = 0;
    for (in_col_index, costs_to_out_rows) in cost_matrix.iter().enumerate() {
        let in_col_cost =
            cost.first_row(graph, problem, &valid_rows[in_col_index], &in_column_edges);
        for (out_row_index, &cost_to_out_row) in costs_to_out_rows.iter().enumerate() {
            let total = cost_to_out_row + in_col_cost;
            if total < min_value {
                min_value = total;
                min_in_col_index = in_col_index;
                min_out_row_index = out_row_index;
            }
        }
    }
    println!("min_value = {}", min_value);
    println!("min_in_col_index = {}", min_in_col_index);
    println!("min_out_row_index = {}", min_out_row_index);

    let path_list = &paths[min_in_col_index][min_out_row_index];
    println!("path_list = {:?}", path_list);
    let mut board_view = BoardView::new(corner_size * 2, corner_size * 2);
    board_view.set_col(0, corner_size, &valid_rows[min_in_col_index], Dir::Left);
    for (row_index, &valid_row_index) in path_list.iter().enumerate() {
        board_view.set_row(row_index, 0, &valid_rows[valid_row_index], Dir::Down);
    }
    board_view.println(true).unwrap();
}

pub struct CornerMatrixResult {
    /// 行の状態の一覧。入力列も出力行も角の行も、この一覧の番号で表す
    pub valid_rows: Vec<Vec<Cell>>,
    /// cost_matrix[入力列][出力行] は、その入力列から出力行までの角の最小コスト
    pub cost_matrix: Matrix,
    /// paths[入力列][出力行] は、そのときの角の行の番号の列（上から順。最後が出力行）。つながらなければ空
    pub paths: Vec<Vec<Vec<usize>>>,
    /// 角で積む行の数。Grid は corner_size（出力行は角の最後の行）、King は corner_size + 1（出力行は角のすぐ下の行）
    pub rows_in_corner: usize,
}

/// 幅 corner_size の角の遷移マトリクスの計算
pub fn corner_transition_matrix(
    corner_size: usize,
    graph: Graph,
    problem: Problem,
    cost: Cost,
    verbose: bool,
) -> CornerMatrixResult {
    assert!(corner_size >= 2, "角の一辺は 2 以上");
    match graph {
        Graph::Grid => corner_transition_matrix_grid(corner_size, problem, cost, verbose),
        Graph::King => corner_transition_matrix_king(corner_size, problem, cost, verbose),
    }
}

/// 角の遷移コストの計算に使う、左上隅の角の四方
const CORNER_EDGES: Edges = Edges {
    top: true,
    bottom: false,
    left: true,
    right: false,
};

/// Grid の角。転送行列の DP で、出力行は角の最後の行
fn corner_transition_matrix_grid(
    corner_size: usize,
    problem: Problem,
    cost: Cost,
    verbose: bool,
) -> CornerMatrixResult {
    // 角の入力列(左上隅の角の右列の右隣の列)ごとにDPを計算
    let mut matrix = vec![];
    let mut paths = vec![];
    let dp_config = init_dp(
        corner_size,
        corner_size,
        Graph::Grid,
        problem,
        cost,
        CORNER_EDGES,
        verbose,
    );
    if verbose {
        println!("dp_config = {}", dp_config);
    }
    for in_column in &dp_config.valid_rows {
        let result = calc_dp(
            &dp_config,
            Some(&|row_index, prev_row, next_row| {
                add_cost_row_for_in_column(row_index, prev_row, next_row, in_column, problem)
            }),
            verbose,
        );
        if verbose {
            print_dp_result(&dp_config, &result);
        }
        let costs_to_out_rows = result.dp_history.last().unwrap().clone();
        // 出力行ごとに、DP の経路を逆にたどって角の行の列にする
        let paths_to_out_rows = costs_to_out_rows
            .iter()
            .enumerate()
            .map(|(out_index, &cost_to_out_row)| {
                if cost_to_out_row >= INF {
                    vec![]
                } else {
                    dp_path(&result.transition_path, out_index)
                }
            })
            .collect();
        matrix.push(costs_to_out_rows);
        paths.push(paths_to_out_rows);
    }

    CornerMatrixResult {
        valid_rows: dp_config.valid_rows,
        cost_matrix: Matrix(matrix),
        paths,
        rows_in_corner: corner_size,
    }
}

/// DP の経路 transition_path を、出力行 out_index から逆にたどって、角の行を上から順に返す
fn dp_path(transition_path: &[Vec<usize>], out_index: usize) -> Vec<usize> {
    let mut index = out_index;
    let mut path = vec![index];
    for transition in transition_path.iter().rev() {
        index = transition[index];
        path.push(index);
    }
    path.reverse();
    path
}

fn add_cost_row_for_in_column(
    row_index: usize,
    prev_row: Option<&[Cell]>,
    next_row: &[Cell],
    in_column: &[Cell],
    problem: Problem,
) -> i64 {
    let in_column_cell = in_column[row_index];
    let next_row_cell = *next_row.last().unwrap();

    let mut stones_around_next_row_cell = 0;
    let mut cost = 0;

    // InColumn側のコスト
    if in_column_cell == Cell::Stone {
        if next_row_cell == Cell::Stone {
            return 2;
        }
    } else if in_column_cell == Cell::Ok {
        if next_row_cell == Cell::Stone {
            if problem == Problem::Fair2 {
                return INF;
            } else {
                cost += 1;
            }
        }
    } else {
        if next_row_cell != Cell::Stone {
            return INF;
        }
    }

    // NextRow側のコスト
    if next_row_cell != Cell::Stone {
        // next_row_cellが石でない場合は周りの石の数をカウント
        if let Some(prev_row) = prev_row
            && *prev_row.last().unwrap() == Cell::Stone
        {
            stones_around_next_row_cell += 1;
        }
        if in_column_cell == Cell::Stone {
            stones_around_next_row_cell += 1;
        }
        if next_row[next_row.len() - 2] == Cell::Stone {
            stones_around_next_row_cell += 1;
        }

        if next_row_cell == Cell::Ok {
            // NextRowがOkの場合の処理
            if stones_around_next_row_cell < 2 {
                // 周りの石の数が2未満の場合は不正
                return INF;
            } else if stones_around_next_row_cell == 3 {
                // 周りの石の数が3の場合の処理
                if problem == Problem::Fair2 {
                    // Fair2の場合は不正
                    return INF;
                } else {
                    // Dom2の場合はコストを加算
                    cost += 1;
                }
            }
        } else if next_row_cell == Cell::NeedOne {
            // NextRowがNeedOneの場合は周りの石の数が1でなければ不正
            if stones_around_next_row_cell != 1 {
                return INF;
            }
        }
    }

    cost
}

/// King の角。入力列ごとに、角の行を上から順に積む DP で、出力行（角のすぐ下の行）までの最小コストを求める
///
/// 角の行の候補と行から行への遷移は Grid と同じ generic な判定で、右が開いているとして作る。
/// 入力列との関係（角の最後の列との隣接）は KingCornerInColumn で判定する。この判定は 2 行前の最後のセルも見るので、
/// DP の状態を (行, 1 つ前の行の最後のセルが石か) にして、1 行前の最後のセルを覚えておく
fn corner_transition_matrix_king(
    corner_size: usize,
    problem: Problem,
    cost: Cost,
    verbose: bool,
) -> CornerMatrixResult {
    let row_edges = CORNER_EDGES.row();
    let valid_rows = list_valid_rows(corner_size, Graph::King, problem, row_edges);
    // transitions[遷移元] は (遷移先, コスト) の一覧
    let transitions =
        build_transition_cost_matrix(&valid_rows, Graph::King, problem, cost, row_edges)
            .to_sparse();
    let first_row_costs: Vec<i64> = valid_rows
        .iter()
        .map(|row| cost.first_row(Graph::King, problem, row, &CORNER_EDGES))
        .collect();
    let state_count = valid_rows.len();
    let rows_in_corner = corner_size + 1;
    if verbose {
        let transition_count: usize = transitions.iter().map(|row| row.len()).sum();
        println!("king corner: 状態の数 {state_count}、遷移の数 {transition_count}、角の行数 {rows_in_corner}");
    }

    let mut matrix = Vec::with_capacity(state_count);
    let mut paths = Vec::with_capacity(state_count);
    for (in_index, in_column) in valid_rows.iter().enumerate() {
        let in_column_check = KingCornerInColumn {
            in_column,
            row_edges,
            problem,
            cost,
        };
        let (costs_to_out_rows, paths_to_out_rows) = king_corner_dp(
            &valid_rows,
            &transitions,
            &first_row_costs,
            &in_column_check,
            rows_in_corner,
        );
        if verbose {
            eprint!("\r入力列 {}/{state_count}", in_index + 1);
        }
        matrix.push(costs_to_out_rows);
        paths.push(paths_to_out_rows);
    }
    if verbose {
        eprintln!();
    }

    CornerMatrixResult {
        valid_rows,
        cost_matrix: Matrix(matrix),
        paths,
        rows_in_corner,
    }
}

/// King の角の DP の状態。行の番号と、その 1 つ上の行の最後のセルが石か
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct KingCornerState {
    row_index: usize,
    prev_last_is_stone: bool,
}

/// King の角の DP で、ある状態までの最小コストと、そのときの 1 つ前の状態（1 行目なら None）
#[derive(Clone, Copy, Debug)]
struct KingCornerStep {
    cost: i64,
    from: Option<KingCornerState>,
}

impl KingCornerStep {
    const UNREACHABLE: KingCornerStep = KingCornerStep {
        cost: INF,
        from: None,
    };
}

/// 入力列 in_column_check.in_column を固定して、角の行を上から rows_in_corner 行積む DP
///
/// 戻り値は、出力行ごとの最小コストと、そのときの角の行の番号の列（つながらなければ空）
fn king_corner_dp(
    valid_rows: &[Vec<Cell>],
    transitions: &SparseMatrix,
    first_row_costs: &[i64],
    in_column_check: &KingCornerInColumn,
    rows_in_corner: usize,
) -> (Vector, Vec<Vec<usize>>) {
    let state_count = valid_rows.len();
    let last_col = valid_rows[0].len() - 1;
    // layers[r][行の番号][1 つ上の行の最後のセルが石なら 1]
    let mut layers: Vec<Vec<[KingCornerStep; 2]>> = Vec::with_capacity(rows_in_corner);

    // 1 行目。1 つ上の行はないので、石かどうかは 0 に入れる
    let mut first_layer = vec![[KingCornerStep::UNREACHABLE; 2]; state_count];
    for (row_index, &first_row_cost) in first_row_costs.iter().enumerate() {
        if first_row_cost >= INF {
            continue;
        }
        let in_column_cost = in_column_check.cost_for_row(0, None, None, &valid_rows[row_index]);
        if in_column_cost >= INF {
            continue;
        }
        first_layer[row_index][0] = KingCornerStep {
            cost: first_row_cost + in_column_cost,
            from: None,
        };
    }
    layers.push(first_layer);

    // 2 行目から出力行まで
    for r in 1..rows_in_corner {
        let mut layer = vec![[KingCornerStep::UNREACHABLE; 2]; state_count];
        for (prev_index, steps) in layers[r - 1].iter().enumerate() {
            let prev_row = &valid_rows[prev_index];
            let prev_last_is_stone = prev_row[last_col] == Cell::Stone;
            for (prev_prev_last_is_stone, step) in [false, true].into_iter().zip(steps) {
                if step.cost >= INF {
                    continue;
                }
                // 2 行前の最後のセル。2 行目にはない
                let two_rows_up_last_is_stone = (r >= 2).then_some(prev_prev_last_is_stone);
                for &(next_index, transition_cost) in &transitions[prev_index] {
                    let in_column_cost = in_column_check.cost_for_row(
                        r,
                        two_rows_up_last_is_stone,
                        Some(prev_row),
                        &valid_rows[next_index],
                    );
                    if in_column_cost >= INF {
                        continue;
                    }
                    let total = step.cost + transition_cost + in_column_cost;
                    let reached = &mut layer[next_index][usize::from(prev_last_is_stone)];
                    if total < reached.cost {
                        *reached = KingCornerStep {
                            cost: total,
                            from: Some(KingCornerState {
                                row_index: prev_index,
                                prev_last_is_stone: prev_prev_last_is_stone,
                            }),
                        };
                    }
                }
            }
        }
        layers.push(layer);
    }

    // 出力行ごとの最小コストと、逆にたどった経路
    let mut costs_to_out_rows = Vec::with_capacity(state_count);
    let mut paths_to_out_rows = Vec::with_capacity(state_count);
    for (out_index, steps) in layers[rows_in_corner - 1].iter().enumerate() {
        let (best_prev_last_is_stone, best_step) = [false, true]
            .into_iter()
            .zip(steps)
            .min_by_key(|(_, step)| step.cost)
            .unwrap();
        costs_to_out_rows.push(best_step.cost);
        if best_step.cost >= INF {
            paths_to_out_rows.push(vec![]);
            continue;
        }
        let mut path = vec![out_index];
        let mut state = KingCornerState {
            row_index: out_index,
            prev_last_is_stone: best_prev_last_is_stone,
        };
        for layer in layers[1..].iter().rev() {
            state = layer[state.row_index][usize::from(state.prev_last_is_stone)]
                .from
                .unwrap();
            path.push(state.row_index);
        }
        path.reverse();
        paths_to_out_rows.push(path);
    }
    (Vector(costs_to_out_rows), paths_to_out_rows)
}

/// King の角で、入力列と角の最後の列の関係を判定する
struct KingCornerInColumn<'a> {
    in_column: &'a [Cell],
    row_edges: RowEdges,
    problem: Problem,
    cost: Cost,
}

impl KingCornerInColumn<'_> {
    /// 入力列のセル。角の行の範囲 0..corner_size の外（出力行の右隣は盤の内側）なら None
    fn in_column_cell(&self, row: usize) -> Option<Cell> {
        self.in_column.get(row).copied()
    }

    /// 入力列の行 row のセルが石なら 1。row が範囲の外（上端の外や、出力行の右隣）なら 0
    fn in_column_stone(&self, row: Option<usize>) -> usize {
        match row.and_then(|row| self.in_column_cell(row)) {
            Some(Cell::Stone) => 1,
            _ => 0,
        }
    }

    /// 行 r に next_row を置くときの、入力列との関係で決まる判定と、loss の追加分。合わなければ INF
    ///
    /// prev_row は 1 つ上の行（1 行目なら None）、two_rows_up_last_is_stone は 2 つ上の行の最後のセルが石か（なければ None）。
    /// 角の最後の列（入力列の左隣）のセルを x_r と書く。generic な判定とコストは、右が開いているとして
    /// 入力列の位置を「まだ分からない」にしているので、ここで入力列の実際のセルに置き換える
    fn cost_for_row(
        &self,
        r: usize,
        two_rows_up_last_is_stone: Option<bool>,
        prev_row: Option<&[Cell]>,
        next_row: &[Cell],
    ) -> i64 {
        let corner_size = self.in_column.len();
        let last_col = corner_size - 1;
        let is_stone = |cell: Cell| usize::from(cell == Cell::Stone);
        // 出力行（r = corner_size）の右隣は盤の内側で、まだ分からない
        let right_of_row_r_is_unknown = usize::from(r == corner_size);

        let mut extra_loss: usize = 0;

        // 1. 入力列の 1 つ上のセル in_column[r−1] が、角の最後の列（x_{r−2}, x_{r−1}, x_r）に求める石
        if let Some(prev_row) = prev_row
            && let Some(in_column_cell) = self.in_column_cell(r - 1)
            && in_column_cell != Cell::Stone
        {
            let stones = two_rows_up_last_is_stone.map_or(0, usize::from)
                + is_stone(prev_row[last_col])
                + is_stone(next_row[last_col]);
            let needed = in_column_cell.needed_from_next_row();
            let fits = match self.problem {
                Problem::Fair2 => stones == needed,
                Problem::Dom2 => stones >= needed,
            };
            if !fits {
                return INF;
            }
            extra_loss += stones - needed;
        }

        // 2. x_r が石でないとき、周りの石に入力列の分（上の行の in_column[r−1] と同じ行の in_column[r]）を足して、
        //    状態をきっちり合わせる。generic は右の 2 つを「分からない」にしていた
        if next_row[last_col] != Cell::Stone {
            let generic_stones = prev_row.map_or(0, |prev_row| {
                count_stones_and_unknown(
                    prev_row,
                    last_col,
                    Graph::King.adjacent_row_column_offsets(),
                    self.row_edges,
                )
                .stones
            }) + count_stones_and_unknown(
                next_row,
                last_col,
                &SAME_ROW_COLUMN_OFFSETS,
                self.row_edges,
            )
            .stones;
            let around = StonesAndUnknown {
                stones: generic_stones
                    + self.in_column_stone(r.checked_sub(1))
                    + self.in_column_stone(Some(r)),
                unknown: right_of_row_r_is_unknown,
            };
            if !cell_state_fits(self.problem, next_row[last_col], around) {
                return INF;
            }
            // 過剰な支配のうち、generic で数えていない分
            extra_loss += around.stones.saturating_sub(2) - generic_stones.saturating_sub(2);
        }

        // 3. 1 つ上の行の x_{r−1} が石でないとき、次の行（行 r）から必要な石に、同じ行の in_column[r] を足して確かめる
        if let Some(prev_row) = prev_row
            && prev_row[last_col] != Cell::Stone
        {
            let needed = prev_row[last_col].needed_from_next_row();
            let generic_stones = count_stones_and_unknown(
                next_row,
                last_col,
                Graph::King.adjacent_row_column_offsets(),
                self.row_edges,
            )
            .stones;
            let in_next_row = StonesAndUnknown {
                stones: generic_stones + self.in_column_stone(Some(r)),
                unknown: right_of_row_r_is_unknown,
            };
            if !next_row_fits_need(self.problem, needed, in_next_row) {
                return INF;
            }
            extra_loss +=
                in_next_row.stones.saturating_sub(needed) - generic_stones.saturating_sub(needed);
        }

        // 4. x_r が石なら、入力列の石（上・同じ行・下）と隣り合う組ごとに 2
        if next_row[last_col] == Cell::Stone {
            let in_column_stones = self.in_column_stone(r.checked_sub(1))
                + self.in_column_stone(Some(r))
                + self.in_column_stone(Some(r + 1));
            extra_loss += 2 * in_column_stones;
        }

        match self.cost {
            Cost::Loss => extra_loss as i64,
            Cost::Stones => 0,
        }
    }
}
