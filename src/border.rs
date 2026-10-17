//! 縁（盤の端から幅 corner_size の部分。四隅の角と 4 本の帯）の最小 loss の計算
//!
//! 角（corner）と帯（band）の計算を組み合わせて、盤を一周する

use crate::{
    corner::{CornerMatrixResult, corner_transition_matrix},
    cost::{Cost, stone_number},
    display::{BoardView, Dir, print_row},
    matrix::{Matrix, compose_with_via},
    rules::{Problem, RowEdges, list_valid_rows},
    transfer::build_transition_cost_matrix,
};

/// 幅 corner_size の縁を一周したときの最小コストの計算。盤は rows × cols、corner_size は角の一辺
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

    // 帯の遷移コスト行列（左端が盤の端）
    let edges = RowEdges {
        left: true,
        right: false,
    };
    let band_rows = list_valid_rows(corner_size, edges);
    let band_matrix = build_transition_cost_matrix(&band_rows, problem, cost, edges);
    // 角の遷移コスト行列
    let CornerMatrixResult {
        valid_rows,
        cost_matrix: corner_matrix,
        transition_path_list,
    } = corner_transition_matrix(corner_size, problem, cost, verbose);

    // 左上の角から反時計回りに、帯と角を交互につないで一周する
    let row_band_len = rows - corner_size * 2;
    let col_band_len = cols - corner_size * 2;
    let mut border_matrix = corner_matrix.clone();
    let mut via_matrix_list = vec![];
    let mut compose = |next: &Matrix| {
        let (composed, via) = compose_with_via(&border_matrix, next);
        border_matrix = composed;
        via_matrix_list.push(via);
    };
    // 左帯 → 左下の角 → 下帯 → 右下の角 → 右帯 → 右上の角 → 上帯（左上の角は初期状態）
    let band_lens = [row_band_len, col_band_len, row_band_len, col_band_len];
    for (i, band_len) in band_lens.into_iter().enumerate() {
        for _ in 0..band_len {
            compose(&band_matrix);
        }
        if i < band_lens.len() - 1 {
            compose(&corner_matrix);
        }
    }

    // 初期列と最後の列が一致してるものを選択
    let (min_index, min_cost) = (0..valid_rows.len())
        .map(|i| (i, border_matrix[i][i]))
        .min_by_key(|&(_, value)| value)
        .unwrap();
    let min_stone = stone_number(Cost::Loss, min_cost, rows, cols);

    // 経路を調べる
    let mut prev_index = min_index;
    let mut path_list = vec![min_index];
    for via_matrix in via_matrix_list.iter().rev() {
        prev_index = via_matrix[min_index][prev_index].unwrap();
        path_list.push(prev_index);
    }
    path_list.reverse();
    if verbose {
        println!("path_list = {:?}", path_list);
    }

    // 経路の行を盤に置く。辺ごとに、角の行に続けて帯の行を、辺の始まりから順に置く
    let sides = [
        ("左辺（左上の角と左帯）", Dir::Down, row_band_len),
        ("下辺（左下の角と下帯）", Dir::Right, col_band_len),
        ("右辺（右下の角と右帯）", Dir::Up, row_band_len),
        ("上辺（右上の角と上帯）", Dir::Left, col_band_len),
    ];
    let mut board_view = BoardView::new(rows, cols);
    // 角の最後の行（出力行）の、path_list での位置
    let mut p = 0;
    for (name, dir, band_len) in sides {
        // 角の入力列は、その 1 つ前の行。左上の角では一周した最後の行
        let in_index = if p == 0 {
            *path_list.last().unwrap()
        } else {
            path_list[p - 1]
        };
        // 角の行（最後が出力行）と、それに続く帯の行
        let mut side_rows = corner_path(&transition_path_list[in_index], path_list[p]);
        side_rows.extend(&path_list[p + 1..=p + band_len]);
        if verbose {
            println!("{name}");
        }
        for (k, &index) in side_rows.iter().enumerate() {
            let row = &valid_rows[index];
            if verbose {
                print_row(row);
            }
            for (j, &cell) in row.iter().enumerate() {
                let (r, c) = side_position(dir, k, j, rows, cols);
                board_view.set(r, c, cell, dir);
            }
        }
        p += band_len + 1;
    }

    // 結論
    board_view.println(verbose).unwrap();
    println!("min_index = {min_index}");
    println!("min_cost = {min_cost}");
    println!("min_stone = {min_stone}");
}

/// dir の向きに積む辺で、辺の始まりから k 行目、縁から j 番目のセルの盤上の位置 (行, 列)
fn side_position(dir: Dir, k: usize, j: usize, rows: usize, cols: usize) -> (usize, usize) {
    match dir {
        // 左辺: 上から下へ積む。縁は左端
        Dir::Down => (k, j),
        // 下辺: 左から右へ積む。縁は下端
        Dir::Right => (rows - 1 - j, k),
        // 右辺: 下から上へ積む。縁は右端
        Dir::Up => (rows - 1 - k, cols - 1 - j),
        // 上辺: 右から左へ積む。縁は上端
        Dir::Left => (j, cols - 1 - k),
    }
}

/// 角の DP の経路 transition_path を、出力行 out_index から逆にたどって、角の行を上から順に返す
fn corner_path(transition_path: &[Vec<usize>], out_index: usize) -> Vec<usize> {
    let mut index = out_index;
    let mut path = vec![index];
    for transition in transition_path.iter().rev() {
        index = transition[index];
        path.push(index);
    }
    path.reverse();
    path
}
