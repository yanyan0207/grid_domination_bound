//! 縁（盤の端から幅 corner_size の部分。四隅の角と 4 本の帯）の最小 loss の計算
//!
//! 角（corner）と帯（band）の計算を組み合わせて、盤を一周する

use crate::{
    board::{calc_dp, init_dp},
    cost::{Cost, stone_number},
    matrix::{Matrix, compose},
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
    let corner_matrix = corner_transition_costs(corner_size, problem, cost, verbose);

    // 左右の帯のマトリクス
    let column_band_matrix = if rows > corner_size * 2 {
        let mut work = transition_matrix.clone();
        for _ in 0..(rows - corner_size * 2 - 1) {
            work = compose(&work, &transition_matrix);
        }
        Some(work)
    } else {
        None
    };

    // 上下の帯のマトリクス
    let row_band_matrix = if cols > corner_size * 2 {
        let mut work = transition_matrix.clone();
        for _ in 0..(cols - corner_size * 2 - 1) {
            work = compose(&work, &transition_matrix);
        }
        Some(work)
    } else {
        None
    };

    // 初期状態は左上の角の遷移コスト行列
    let mut border_matrix = corner_matrix.clone();

    // 左端の帯
    if let Some(column_band_matrix) = &column_band_matrix {
        border_matrix = compose(&border_matrix, column_band_matrix);
    }
    // 左下の角
    border_matrix = compose(&border_matrix, &corner_matrix);
    // 下端の帯
    if let Some(row_band_matrix) = &row_band_matrix {
        border_matrix = compose(&border_matrix, row_band_matrix);
    }
    // 右下の角
    border_matrix = compose(&border_matrix, &corner_matrix);
    // 右端の帯
    if let Some(column_band_matrix) = &column_band_matrix {
        border_matrix = compose(&border_matrix, column_band_matrix);
    }
    // 右上の角
    border_matrix = compose(&border_matrix, &corner_matrix);
    // 上端の帯
    if let Some(row_band_matrix) = &row_band_matrix {
        border_matrix = compose(&border_matrix, row_band_matrix);
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
    let mut path = vec![min_index];
    println!("\n[corner_matrix] =\n{corner_matrix}");
    if let Some(row_band_matrix) = &row_band_matrix {
        println!("\n[row_band_matrix] =\n{row_band_matrix}");
    }
    if let Some(column_band_matrix) = &column_band_matrix {
        println!("\n[column_band_matrix] =\n{column_band_matrix}");
    }

    fn search(
        path: &mut Vec<usize>,
        corner_matrix: &Matrix,
        band_matrix: &Matrix,
        column_matrix: &Matrix,
        current_cost: i64,
        min_cost: i64,
    ) -> bool {
        let last_index = *path.last().unwrap();
        if path.len() == 8 {
            let current_cost = current_cost + band_matrix[last_index][path[0]];
            assert!(current_cost >= min_cost);
            if current_cost == min_cost {
                path.push(path[0]); // 経路を閉じる
                return true;
            } else {
                return false;
            }
        } else {
            let &matrix = if path.len() % 2 == 1 {
                &corner_matrix
            } else if path.len() == 4 {
                &band_matrix
            } else {
                &column_matrix
            };

            // コーナーマトリクスの流れを追う
            for (i, cost) in matrix[last_index]
                .iter()
                .enumerate()
                .filter(|&(_, c)| current_cost + *c <= min_cost)
            {
                path.push(i);
                if search(
                    path,
                    corner_matrix,
                    band_matrix,
                    column_matrix,
                    current_cost + *cost,
                    min_cost,
                ) {
                    return true;
                }
                path.pop();
            }
        }
        false
    }

    let found = search(
        &mut path,
        &corner_matrix,
        &row_band_matrix.unwrap(),
        &column_band_matrix.unwrap(),
        0,
        min_cost,
    );
    assert!(found);

    println!("path = {:?}", path);

    // 経路を探すためにDPを行う
}

/// 幅 corner_size の角の最小コストの計算（未実装）
pub fn corner_transition_costs(
    corner_size: usize,
    problem: Problem,
    cost: Cost,
    verbose: bool,
) -> Matrix {
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
    }
    Matrix(matrix)
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
    let mut stones_around_in_column_cell = 0;
    let mut stones_around_next_row_cell = 0;
    if in_column_cell != Cell::Stone {
        // in_column_cellが石でない場合は周りの石の数をカウント
        if row_index > 0 && in_column[row_index - 1] == Cell::Stone {
            stones_around_in_column_cell += 1;
        }
        if row_index < in_column.len() - 1 && in_column[row_index + 1] == Cell::Stone {
            stones_around_in_column_cell += 1;
        }
        if next_row_cell == Cell::Stone {
            stones_around_in_column_cell += 1;
        }
    }
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

    // 両方が石の場合はコスト2を返す
    if next_row_cell == Cell::Stone && in_column_cell == Cell::Stone {
        return 2;
    }

    // NeedOneセルの周囲の石の数が1でない場合はINFを返す
    let need_one_stones = if row_index == in_column.len() - 1 {
        vec![0, 1]
    } else {
        vec![1]
    };
    if in_column_cell == Cell::NeedOne && !need_one_stones.contains(&stones_around_in_column_cell) {
        return INF;
    }
    if next_row_cell == Cell::NeedOne && !need_one_stones.contains(&stones_around_next_row_cell) {
        return INF;
    }

    if problem == Problem::Fair2 {
        // Okセルの周囲の石の数が2でない場合はINFを返す
        let ok_stones = if row_index == in_column.len() - 1 {
            vec![1, 2]
        } else {
            vec![2]
        };
        if next_row_cell == Cell::Ok && !ok_stones.contains(&stones_around_next_row_cell) {
            return INF;
        }
        if in_column_cell == Cell::Ok && !ok_stones.contains(&stones_around_in_column_cell) {
            return INF;
        }
        0
    } else {
        let mut cost = 0;
        // Okセルの周囲の石の数が2でない場合、コストを計算する
        if next_row_cell == Cell::Ok {
            // next_row_cellがOkセルの場合の石の数のチェック
            if stones_around_next_row_cell == 0
                || (row_index < in_column.len() - 1 && stones_around_next_row_cell == 1)
            {
                return INF;
            }
            if stones_around_next_row_cell == 3 {
                cost += 1;
            }
        }
        if in_column_cell == Cell::Ok {
            if stones_around_in_column_cell == 0
                || (row_index < in_column.len() - 1 && stones_around_in_column_cell == 1)
            {
                return INF;
            }
            if stones_around_in_column_cell == 3 {
                cost += 1;
            }
        }
        cost
    }
}
