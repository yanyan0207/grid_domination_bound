//! 問題ごとの規則（列を並べられるかの判定）

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

/// セルの状態。前の行と同じ行の石を数えた時点で、次の行に何が必要かを表す
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// 石
    Stone,
    /// 周りに必要な石がある
    Ok,
    /// 周りに石が1つ必要
    NeedOne,
}

/// 指定された幅のすべての可能な行のリストから、問題に従って有効な行だけを返す
pub fn list_valid_rows(width: usize, top_is_edge: bool, bottom_is_edge: bool) -> Vec<Vec<Cell>> {
    let mut valid_rows: Vec<Vec<Cell>> = Vec::new();
    for i in 0..(3u64.pow(width as u32)) {
        let mut row = Vec::with_capacity(width);
        let mut n = i;
        for _ in 0..width {
            row.push(match n % 3 {
                0 => Cell::Stone,
                1 => Cell::Ok,
                2 => Cell::NeedOne,
                _ => unreachable!(),
            });
            n /= 3;
        }
        if is_valid_row(&row, top_is_edge, bottom_is_edge, false) {
            valid_rows.push(row);
        }
    }
    valid_rows
}

/// 遷移が有効かどうかを判定
pub fn is_valid_transition(
    prev_row: &[Cell],
    next_row: &[Cell],
    problem: Problem,
    top_is_edge: bool,
    bottom_is_edge: bool,
) -> bool {
    // 前行を確認する
    for i in 0..prev_row.len() {
        // 次の石との関係を確認
        if prev_row[i] == Cell::NeedOne && next_row[i] != Cell::Stone {
            return false;
        }
        if problem == Problem::Fair2 && prev_row[i] == Cell::Ok && next_row[i] == Cell::Stone {
            return false;
        }
    }
    // 次行を確認する
    for i in 0..next_row.len() {
        if next_row[i] == Cell::Stone {
            continue;
        }
        // 周りの石の数を計算
        let mut cnt = 0;
        if i > 0 && next_row[i - 1] == Cell::Stone {
            cnt += 1;
        }
        if i < next_row.len() - 1 && next_row[i + 1] == Cell::Stone {
            cnt += 1;
        }
        if prev_row[i] == Cell::Stone {
            cnt += 1;
        }
        // NeedOne のセルの周りに石が二つ以上ある場合は無効
        if next_row[i] == Cell::NeedOne && cnt >= 2 {
            return false;
        }
        // NeedOne のセルの周りに石が一つもない場合は無効
        if next_row[i] == Cell::NeedOne
            && cnt == 0
            && !((i == 0 && !top_is_edge) || (i == next_row.len() - 1 && !bottom_is_edge))
        {
            return false;
        }
        // Ok のセルの周りに石が一つもないか、端が開いてなくて石が1つしかない場合は無効
        if next_row[i] == Cell::Ok
            && (cnt == 0
                || (cnt == 1
                    && !((i == 0 && !top_is_edge) || (i == next_row.len() - 1 && !bottom_is_edge))))
        {
            return false;
        }
        // 2-fairの場合は3つ以上の石が周りにある Ok のセルは無効
        if problem == Problem::Fair2 && next_row[i] == Cell::Ok && cnt >= 3 {
            return false;
        }
    }
    true
}

/// 指定された行が問題に従って有効かどうかを判定
pub fn is_valid_row(
    row: &[Cell],
    top_is_edge: bool,
    bottom_is_edge: bool,
    is_first_row: bool,
) -> bool {
    for (i, cell) in row.iter().enumerate() {
        // 石なら常に有効
        if cell == &Cell::Stone {
            continue;
        }
        // それ以外の場合、周りの石の数をカウント
        let mut cnt = 0;
        if i > 0 && row[i - 1] == Cell::Stone {
            cnt += 1;
        }
        if i < row.len() - 1 && row[i + 1] == Cell::Stone {
            cnt += 1;
        }
        // 周りの空いてるセルの数をカウント
        let mut empty_cnt = 0;
        if !is_first_row {
            empty_cnt += 1;
        }
        if i == 0 && !top_is_edge {
            empty_cnt += 1;
        }
        if i == row.len() - 1 && !bottom_is_edge {
            empty_cnt += 1;
        }

        // OKの場合は周りの石と空いてるセルの合計が2以上でなければ無効
        if cell == &Cell::Ok && cnt + empty_cnt < 2 {
            return false;
        }
        // NeedOne のセル
        if cell == &Cell::NeedOne {
            // 既に二つそろってたらエラー
            if cnt == 2 {
                return false;
            }
            // 周りの石が一つは必要
            if cnt + empty_cnt < 1 {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_valid_rows_width_one() {
        // 幅 1 で両端が盤の端なら、Ok は次の行の石 1 つだけでは足りない
        assert_eq!(
            list_valid_rows(1, true, true),
            vec![vec![Cell::Stone], vec![Cell::NeedOne]]
        );
    }

    #[test]
    fn is_valid_row_first_row_has_no_previous() {
        // 1 行目では前の行がないので、横の石 1 つの Ok は足りない
        let row = [Cell::Ok, Cell::Stone, Cell::Ok];
        assert!(!is_valid_row(&row, true, true, true));
        assert!(is_valid_row(&row, true, true, false));
    }

    #[test]
    fn is_valid_row_need_one_between_stones() {
        // 両隣が石なら、もう石が 2 つあるので NeedOne ではない
        let row = [Cell::Stone, Cell::NeedOne, Cell::Stone];
        assert!(!is_valid_row(&row, true, true, false));
    }

    #[test]
    fn is_valid_transition_need_one_requires_stone() {
        // NeedOne の次は石でなければならない
        let prev = [Cell::NeedOne];
        assert!(is_valid_transition(
            &prev,
            &[Cell::Stone],
            Problem::Dom2,
            true,
            true
        ));
        assert!(!is_valid_transition(
            &prev,
            &[Cell::Ok],
            Problem::Dom2,
            true,
            true
        ));
    }

    #[test]
    fn is_valid_transition_fair_forbids_stone_after_ok() {
        // 2-fair では、石が揃った Ok の次に石を置くと 3 つ目になる
        let prev = [Cell::Ok];
        let next = [Cell::Stone];
        assert!(is_valid_transition(&prev, &next, Problem::Dom2, true, true));
        assert!(!is_valid_transition(
            &prev,
            &next,
            Problem::Fair2,
            true,
            true
        ));
    }
}
