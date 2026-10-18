//! 盤面や行の描画

use std::fmt;

use crate::rules::Cell;

/// セルを表す 1 文字。None（まだ決まっていないセル）は「-」
fn cell_char(cell: Option<Cell>) -> char {
    match cell {
        None => '-',
        Some(Cell::Stone) => '#',
        Some(Cell::Ok) => 'o',
        Some(Cell::NeedOne) => '?',
        Some(Cell::NeedTwo) => '2',
    }
}

/// セルを 1 文字表示する。None（まだ決まっていないセル）は「-」
pub fn print_cell(cell: Option<Cell>) {
    print!("{}", cell_char(cell));
}

/// 行を 1 行表示する
pub fn print_row<T: Copy + Into<Option<Cell>>>(row: &[T]) {
    for &cell in row {
        print_cell(cell.into());
    }
    println!();
}

/// 行を 1 行表示する。verbose のときは step_cost =（その行で増えたコスト, 累計）も表示する
pub fn print_row_with_cost(row: &[Cell], verbose: bool, step_cost: Option<(i64, i64)>) {
    if !verbose {
        return;
    }
    let line: String = row
        .iter()
        .map(|cell| match cell {
            Cell::Stone => '#',
            Cell::Ok => '.',
            Cell::NeedOne if verbose => '?',
            Cell::NeedTwo if verbose => '2',
            Cell::NeedOne | Cell::NeedTwo => '.',
        })
        .collect();
    match step_cost {
        Some((step, total)) if verbose => println!("{line}  +{step} (累計 {total})"),
        _ => println!("{line}"),
    }
}

/// 行を積む方向（次の行が来る向き）。Ok / NeedOne / NeedTwo はこの向きの隣に対する状態
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Dir {
    Down,
    Right,
    Up,
    Left,
}

impl Dir {
    /// 向きを表す 1 文字
    fn arrow(self) -> char {
        match self {
            Dir::Down => 'v',
            Dir::Right => '>',
            Dir::Up => '^',
            Dir::Left => '<',
        }
    }
}

/// 盤全体の表示用。cells[行][列] は（セル, そのセルを積んだ方向）で、まだ決まっていないセルは None
pub struct BoardView {
    cells: Vec<Vec<Option<(Cell, Dir)>>>,
}

impl BoardView {
    /// rows × cols の、全部のセルが未定の盤
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            cells: vec![vec![None; cols]; rows],
        }
    }

    /// (row, col) のセルを、dir の向きに積んだ cell にする
    pub fn set(&mut self, row: usize, col: usize, cell: Cell, dir: Dir) {
        self.cells[row][col] = Some((cell, dir));
    }

    pub fn set_row(&mut self, row: usize, col: usize, cells: &[Cell], dir: Dir) {
        for (i, &c) in cells.iter().enumerate() {
            self.cells[row][col + i] = Some((c, dir));
        }
    }

    pub fn set_col(&mut self, row: usize, col: usize, cells: &[Cell], dir: Dir) {
        for (i, &c) in cells.iter().enumerate() {
            self.cells[row + i][col] = Some((c, dir));
        }
    }
}

/// 1 行ずつ、1 セル 2 文字で表示する。println!("{board_view}") で盤全体が出る
impl BoardView {
    pub fn println(&self, dir: bool) -> fmt::Result {
        let mut cell_line = vec![];
        let mut dir_line = vec![];
        for row in &self.cells {
            for &cell_dir in row {
                if let Some((cell, dir)) = cell_dir {
                    cell_line.push(cell_char(Some(cell)));
                    dir_line.push(dir.arrow());
                } else {
                    cell_line.push('-');
                    dir_line.push('-');
                }
            }
            cell_line.push('\n');
            dir_line.push('\n');
        }
        println!("{}", cell_line.iter().collect::<String>());
        if dir {
            println!("{}", dir_line.iter().collect::<String>());
        }
        Ok(())
    }
}
