//! 縁（盤の端から幅 corner_size の部分。四隅の角と 4 本の帯）の最小 loss の計算
//!
//! 角（corner）と帯（band）の計算を組み合わせて、盤を一周する

use crate::{
    board::{calc_dp, init_dp},
    cost::{Cost, stone_number},
    matrix::{Matrix, compose, compose_with_via},
    rules::{Cell, Edges, Problem, RowEdges, list_valid_rows},
    transfer::{INF, build_transition_cost_matrix},
};

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
        "border matrix: rows = {rows}, cols = {cols}, corner_size = {corner_size}, problem = {problem}, cost = {cost:?}, verbose = {verbose}"
    );

    // 有効行を取得
    let valid_rows = list_valid_rows(
        corner_size,
        RowEdges {
            left: true,
            right: false,
        },
    );
    // 遷移コスト行列を構築
    let transition_matrix = build_transition_cost_matrix(
        &valid_rows,
        problem,
        cost,
        RowEdges {
            left: true,
            right: false,
        },
    );
    // 角の遷移コスト行列を構築
    let (corner_matrix, corner_transition_path_list) =
        corner_transition_costs(corner_size, problem, cost, verbose);

    // 初期状態は左上の角の遷移コスト行列
    let mut border_matrix = corner_matrix.clone();
    let mut via_matrix_list = vec![];

    // 左端の帯
    for _ in 0..(rows - corner_size * 2) {
        let (new_border_matrix, via) = compose_with_via(&border_matrix, &transition_matrix);
        border_matrix = new_border_matrix;
        via_matrix_list.push(via);
    }

    // 左下の角
    let (new_border_matrix, via) = compose_with_via(&border_matrix, &corner_matrix);
    border_matrix = new_border_matrix;
    via_matrix_list.push(via);

    // 下端の帯
    for _ in 0..(cols - corner_size * 2) {
        let (new_border_matrix, via) = compose_with_via(&border_matrix, &transition_matrix);
        border_matrix = new_border_matrix;
        via_matrix_list.push(via);
    }

    // 右下の角
    let (new_border_matrix, via) = compose_with_via(&border_matrix, &corner_matrix);
    border_matrix = new_border_matrix;
    via_matrix_list.push(via);

    // 右端の帯
    for _ in 0..(rows - corner_size * 2) {
        let (new_border_matrix, via) = compose_with_via(&border_matrix, &transition_matrix);
        border_matrix = new_border_matrix;
        via_matrix_list.push(via);
    }

    // 右上の角
    let (new_border_matrix, via) = compose_with_via(&border_matrix, &corner_matrix);
    border_matrix = new_border_matrix;
    via_matrix_list.push(via);

    // 上端の帯
    for _ in 0..(cols - corner_size * 2) {
        let (new_border_matrix, via) = compose_with_via(&border_matrix, &transition_matrix);
        border_matrix = new_border_matrix;
        via_matrix_list.push(via);
    }

    // 初期列と最後の列が一致してるものを選択
    let (min_index, min_cost) = (0..valid_rows.len())
        .into_iter()
        .map(|i| (i, border_matrix[i][i]))
        .min_by_key(|&(_i, val)| val)
        .unwrap();

    println!("min_index = {min_index}, min_cost = {min_cost}");
    let min_stone = stone_number(Cost::Loss, min_cost, rows, cols);
    println!("min_stone = {min_stone}");

    // 経路を調べる
    let mut work = (min_index, min_index);
    let mut path_list = vec![min_index];
    for via_matrix in via_matrix_list.iter().rev() {
        let prev_index = via_matrix[work.0][work.1].unwrap();
        work = (min_index, prev_index);
        path_list.push(prev_index);
    }
    path_list.reverse();
    println!("path_list = {:?}", path_list);

    println!("左上隅");
    println!("左帯");
    let mut index = 0;
    let mut board: Vec<Vec<Option<Cell>>> = vec![vec![None; cols]; rows];
    for row_index in corner_size - 1..rows - corner_size {
        let row = &valid_rows[path_list[index]];
        print_row(row);
        for j in 0..corner_size {
            board[row_index][j] = Some(row[j]);
        }
        index += 1;
    }
    println!("下帯");
    for col_index in corner_size - 1..cols - corner_size {
        let row = &valid_rows[path_list[index]];
        print_row(row);
        for j in 0..corner_size {
            board[rows - 1 - j][col_index] = Some(row[j]);
        }
        index += 1;
    }
    println!("右帯");
    for row_index in (corner_size..rows - corner_size + 1).rev() {
        let row = &valid_rows[path_list[index]];
        print_row(row);
        for j in 0..corner_size {
            board[row_index][cols - 1 - j] = Some(row[j]);
        }
        index += 1;
    }
    println!("上帯");
    for col_index in (corner_size..cols - corner_size + 1).rev() {
        let row = &valid_rows[path_list[index]];
        print_row(row);
        for j in 0..corner_size {
            board[corner_size - 1 - j][col_index] = Some(row[j]);
        }
        index += 1;
    }

    println!("{:?}", board);
    for i in 0..rows {
        for j in 0..cols {
            print_cell(board[i][j]);
        }
        println!();
    }
}

fn print_cell(cell: Option<Cell>) {
    match cell {
        None => print!("-"),
        Some(Cell::Stone) => print!("#"),
        Some(Cell::Ok) => print!("o"),
        Some(Cell::NeedOne) => print!("?"),
    }
}

fn print_row(row: &[Cell]) {
    for &cell in row {
        print_cell(Some(cell));
    }
    println!();
}

/// 幅 corner_size の角の最小コストの計算（未実装）
pub fn corner_transition_costs(
    corner_size: usize,
    problem: Problem,
    cost: Cost,
    verbose: bool,
) -> (Matrix, Vec<Vec<Vec<usize>>>) {
    println!(
        "corner: corner_size = {corner_size}, problem = {problem}, cost = {cost:?}, verbose = {verbose}"
    );

    // 角の遷移コストの計算に左上隅を使用する
    let edges = Edges {
        top: true,
        bottom: false,
        left: true,
        right: false,
    };

    // 有効な行を取得
    let valid_rows = list_valid_rows(corner_size, edges.row());

    // 角の入力列(左上隅の角の右列の右隣の列)ごとにDPを計算
    let mut matrix = vec![];
    let dp_config = init_dp(corner_size, corner_size, problem, cost, edges, verbose);
    let mut transition_path_list = vec![];
    for in_column in valid_rows {
        let result = calc_dp(
            &dp_config,
            Some(&|row_index, prev_row, next_row| {
                add_cost_row_for_in_column(row_index, prev_row, next_row, &in_column, problem)
            }),
            verbose,
        );
        if verbose {
            println!("in_column = {:?}, result = {:?}", in_column, result);
        }
        matrix.push(result.dp_history.last().unwrap().clone());
        transition_path_list.push(result.transition_path);
    }
    (Matrix(matrix), transition_path_list)
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
    }

    if in_column_cell == Cell::Stone {
        if next_row_cell == Cell::Stone {
            return 2;
        }
    }

    // InColumn側のコスト
    if in_column_cell == Cell::Ok {
        if next_row_cell == Cell::Stone {
            if problem == Problem::Fair2 {
                return INF;
            } else {
                cost += 1;
            }
        }
    }
    if in_column_cell == Cell::NeedOne {
        if next_row_cell != Cell::Stone {
            return INF;
        }
    }

    // NextRow側のコスト
    if next_row_cell == Cell::Ok {
        if stones_around_next_row_cell < 2 {
            return INF;
        } else if stones_around_next_row_cell == 3 {
            if problem == Problem::Fair2 {
                return INF;
            } else {
                cost += 1;
            }
        }
    }
    if next_row_cell == Cell::NeedOne {
        if stones_around_next_row_cell != 1 {
            return INF;
        }
    }
    cost
}
