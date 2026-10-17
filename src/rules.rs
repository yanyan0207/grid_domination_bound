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
    /// 周りに石が1つ必要
    NeedOne,
}

/// 指定された幅のすべての可能な行のリストから、問題に従って有効な行だけを返す
pub fn list_valid_rows(width: usize, row_edges: RowEdges) -> Vec<Vec<Cell>> {
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
        // 途中の行として有効か（上下は端でない）
        let edges = Edges {
            top: false,
            bottom: false,
            left: row_edges.left,
            right: row_edges.right,
        };
        if is_valid_row(&row, &edges) {
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
    row_edges: RowEdges,
) -> bool {
    let RowEdges {
        left: left_is_edge,
        right: right_is_edge,
    } = row_edges;
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
            && !((i == 0 && !left_is_edge) || (i == next_row.len() - 1 && !right_is_edge))
        {
            return false;
        }
        // Ok のセルの周りに石が一つもないか、端が開いてなくて石が1つしかない場合は無効
        if next_row[i] == Cell::Ok
            && (cnt == 0
                || (cnt == 1
                    && !((i == 0 && !left_is_edge) || (i == next_row.len() - 1 && !right_is_edge))))
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
pub fn is_valid_row(row: &[Cell], edges: &Edges) -> bool {
    let Edges {
        top: top_is_edge,
        bottom: _bottom_is_edge,
        left: left_is_edge,
        right: right_is_edge,
    } = *edges;
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
        if !top_is_edge {
            empty_cnt += 1;
        }
        if i == 0 && !left_is_edge {
            empty_cnt += 1;
        }
        if i == row.len() - 1 && !right_is_edge {
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

    use std::collections::HashSet;

    use super::*;
    const CELL_LIST: [Cell; 3] = [Cell::Ok, Cell::Stone, Cell::NeedOne];

    #[test]
    fn test_list_valid_rows_2() {
        let rows = list_valid_rows(
            2,
            RowEdges {
                left: true,
                right: true,
            },
        );
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok]));
        assert_eq!(rows.len(), 6);
    }

    #[test]
    fn test_list_valid_rows_2_left_not_edge() {
        let rows = list_valid_rows(
            2,
            RowEdges {
                left: false,
                right: true,
            },
        );
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::NeedOne, Cell::Ok]));
        assert_eq!(rows.len(), 7);
    }

    #[test]
    fn test_list_valid_rows_2_right_not_edge() {
        let rows = list_valid_rows(
            2,
            RowEdges {
                left: true,
                right: false,
            },
        );
        assert!(!rows.contains(&vec![Cell::Ok, Cell::Ok]));
        assert!(!rows.contains(&vec![Cell::Ok, Cell::NeedOne]));
        assert_eq!(rows.len(), 7);
    }

    #[test]
    fn test_list_valid_rows_2_both_not_edge() {
        let rows = list_valid_rows(
            2,
            RowEdges {
                left: false,
                right: false,
            },
        );
        assert_eq!(rows.len(), 9);
    }

    #[test]
    fn test_list_valid_rows_3() {
        let rows = list_valid_rows(
            3,
            RowEdges {
                left: true,
                right: true,
            },
        );
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
        let rows = list_valid_rows(
            3,
            RowEdges {
                left: false,
                right: true,
            },
        );
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
        let rows = list_valid_rows(
            3,
            RowEdges {
                left: true,
                right: false,
            },
        );
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
        let rows = list_valid_rows(
            3,
            RowEdges {
                left: false,
                right: false,
            },
        );
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
        let rows = list_valid_rows(width, RowEdges { left, right });

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
}
