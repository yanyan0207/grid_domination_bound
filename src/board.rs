//! 盤全体（n × m）の最小コストの計算

use crate::{
    rules::{Cell, Problem, is_valid_row, list_valid_rows},
    transfer::{INF, transition_cost_matrix},
};

/// n × m の盤全体の最小コストの計算
pub fn board(rows: usize, cols: usize, problem: Problem) {
    println!("board: rows = {rows}, cols = {cols}, problem = {problem}");

    // 有効な行のリストを取得(左端が端、右端が端)
    let valid_rows = list_valid_rows(rows, true, true);

    // その行のコストを取得
    let row_costs: Vec<i64> = valid_rows
        .iter()
        .map(|row| row.iter().filter(|&&cell| cell == Cell::Stone).count() as i64)
        .collect();

    // 遷移コストを取得
    // コストは遷移先の石の数
    // 作られる遷移コスト行列は、遷移先の行が行、遷移元の行が列となる
    let transition_cost_matrix =
        transition_cost_matrix(&valid_rows, problem, true, true, &row_costs);

    // 遷移コストの計算
    // 1行目の遷移コストを計算
    let mut dp: Vec<i64> = valid_rows
        .iter()
        .zip(row_costs)
        .map(|(row, cost)| {
            if is_valid_row(row, true, true, true) {
                INF
            } else {
                cost
            }
        })
        .collect();
    // 遷移の経路を保存
    let mut transition_path: Vec<Vec<usize>> = (0..valid_rows.len()).map(|i| vec![i]).collect();

    // 残りの行の遷移コストを計算
    for _ in 1..rows {
        // 対象の状態に遷移するためのコストを計算
        for col in 0..cols {
            let new_costs: Vec<i64> = dp
                .iter()
                .zip(&transition_cost_matrix[col])
                .map(|(cost, transition_cost)| cost + transition_cost)
                .collect();
            // 最小値のindexを取得
            let min_index = new_costs
                .iter()
                .enumerate()
                .min_by_key(|&(_, cost)| cost)
                .map(|(index, _)| index)
                .unwrap();
            dp[col] = new_costs[min_index];
            transition_path[col].push(min_index);
        }
    }

    // 最後のパスとして使えないパスをINFにする
    for col in 0..cols {
        if valid_rows[col].contains(&Cell::NeedOne) {
            dp[col] = INF;
        }
    }

    // 最小コストとその経路を出力
    if let Some((min_cost, min_index)) = dp
        .iter()
        .enumerate()
        .min_by_key(|&(_, cost)| cost)
        .map(|(index, &cost)| (cost, index))
    {
        println!("最小コスト: {min_cost}");
        println!("最小コストの経路: {:?}", transition_path[min_index]);
    } else {
        println!("有効な経路が存在しません");
    }
}
