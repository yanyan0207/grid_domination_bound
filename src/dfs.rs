//! 有効な行を 1 つずつ選んで並べ、最小コストを求める（DFS）
use crate::rules::{Cell, Problem, is_valid_row, is_valid_transition};

/// 有効な行 valid_rows を rows 行並べたときの最小コストを DFS で求める（未実装）
///
/// コストは並べ終えた盤面全体（上から順の行）を cost に渡して求める。コストの決め方は呼び出し側に任せる
pub fn dfs(
    valid_rows: &[Vec<Cell>],
    rows: usize,
    problem: Problem,
    cost: impl Fn(&[usize]) -> i64,
) -> (i64, Vec<usize>) {
    println!(
        "dfs: rows = {rows}, valid_count = {}, problem = {problem}",
        valid_rows.len()
    );

    let mut stack = vec![0];

    fn _next(stack: &mut Vec<usize>, valid_cnt: usize) {
        loop {
            // 次の番号を試す
            let length = stack.len();
            let last = stack[length - 1];
            if last < valid_cnt - 1 {
                stack[length - 1] += 1;
                break;
            }

            // 次の番号が valid_cnt - 1 以上の場合は前に戻る
            stack.pop();

            // 空になったら終了
            if stack.is_empty() {
                break;
            }
        }
    }

    let mut min_cost = i64::MAX;
    let mut min_pattern = vec![];
    while !stack.is_empty() {
        // 問題ないか確認
        let mut valid;
        if stack.len() == 1 {
            // 最初の行の場合は行単体の有効性を確認
            valid = is_valid_row(&valid_rows[stack[0]], true, true, true);
        } else {
            let prev_row = &valid_rows[stack[stack.len() - 2]];
            let next_row = &valid_rows[stack[stack.len() - 1]];
            // 2 行目以降の場合は前の行との遷移の有効性を確認
            valid = is_valid_transition(prev_row, next_row, problem, true, true);
            if valid && stack.len() == rows {
                // 最後の行の場合は、行内にまだ必要なセルが残っていないか確認
                valid = !next_row.contains(&Cell::NeedOne);
            }
        }

        // 妥当であれば枝刈りを確認
        if valid {
            let current_cost = cost(&stack);
            if current_cost >= min_cost {
                valid = false;
            }
        }
        // valid が false なら次の行を試す
        if !valid {
            _next(&mut stack, valid_rows.len());
            continue;
        }

        // valid が true なら次の行を追加
        if stack.len() < rows {
            stack.push(0);
            continue;
        }

        // 最後の行まで有効に並べられた場合はコストを計算
        let current_cost = cost(&stack);
        if current_cost < min_cost {
            min_cost = current_cost;
            min_pattern = stack.clone();
        }
        _next(&mut stack, valid_rows.len());
    }
    (min_cost, min_pattern)
}
