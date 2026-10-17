//! 角（盤の隅の、一辺 corner_size の正方形）の計算

use crate::board::{calc_dp, init_dp};
use crate::display::{BoardView, Dir};
use crate::matrix::Matrix;
use crate::rules::{Cell, Edges};
use crate::{cost::Cost, rules::Problem, transfer::INF};

/// 一辺 corner_size の角の計算（未実装）
pub fn corner(corner_size: usize, problem: Problem, cost: Cost, verbose: bool) {
    println!(
        "corner: corner_size = {corner_size}, problem = {problem}, cost = {cost:?}, verbose = {verbose}"
    );

    let CornerMatrixResult {
        valid_rows,
        cost_matrix,
        transition_path_list,
    } = corner_transition_matrix(corner_size, problem, cost, verbose);

    // 最小経路を探す
    let mut min_value = INF;
    let mut min_in_col_index = 0;
    let mut min_out_row_index = 0;
    for (in_col_index, dp) in cost_matrix.0.iter().enumerate() {
        let in_col_cost = Cost::Loss.first_row(
            &valid_rows[in_col_index],
            &Edges {
                top: false,
                bottom: false,
                left: true,
                right: false,
            },
        );
        for (out_row_index, value) in dp.iter().enumerate() {
            if *value < min_value + in_col_cost {
                min_value = *value + in_col_cost;
                min_in_col_index = in_col_index;
                min_out_row_index = out_row_index;
            }
        }
    }
    if verbose {
        println!("transition_path_list = {:?}", transition_path_list);
    }
    println!("min_value = {}", min_value);
    println!("min_in_col_index = {}", min_in_col_index);
    println!("min_out_row_index = {}", min_out_row_index);

    let min_transition_path = &transition_path_list[min_in_col_index];
    let work = min_out_row_index;
    let mut path_list = vec![work];
    if verbose {
        println!("min_transition_path = {:?}", min_transition_path);
    }
    for min_transition in min_transition_path.iter().rev() {
        let work = min_transition[work];
        path_list.push(work);
    }
    path_list.reverse();
    println!("path_list = {:?}", path_list);
    let mut board_view = BoardView::new(corner_size * 2, corner_size * 2);
    board_view.set_col(0, corner_size, &valid_rows[min_in_col_index], Dir::Left);
    for (row_index, valid_row_index) in path_list.into_iter().enumerate() {
        board_view.set_row(row_index, 0, &valid_rows[valid_row_index], Dir::Down);
    }
    board_view.println(true).unwrap();
}

pub struct CornerMatrixResult {
    pub valid_rows: Vec<Vec<Cell>>,
    pub cost_matrix: Matrix,
    pub transition_path_list: Vec<Vec<Vec<usize>>>,
}

/// 幅 corner_size の角の遷移マトリクスの計算
pub fn corner_transition_matrix(
    corner_size: usize,
    problem: Problem,
    cost: Cost,
    verbose: bool,
) -> CornerMatrixResult {
    // 角の遷移コストの計算に左上隅を使用する
    let edges = Edges {
        top: true,
        bottom: false,
        left: true,
        right: false,
    };

    // 角の入力列(左上隅の角の右列の右隣の列)ごとにDPを計算
    let mut matrix = vec![];
    let dp_config = init_dp(corner_size, corner_size, problem, cost, edges, verbose);
    let mut transition_path_list = vec![];
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
        matrix.push(result.dp_history.last().unwrap().clone());
        transition_path_list.push(result.transition_path);
    }
    return CornerMatrixResult {
        valid_rows: dp_config.valid_rows,
        cost_matrix: Matrix(matrix),
        transition_path_list,
    };
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
    } else if in_column_cell == Cell::NeedOne {
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
