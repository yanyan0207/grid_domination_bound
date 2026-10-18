//! 縁（盤の端から幅 corner_size の部分。四隅の角と 4 本の帯）の最小 loss の計算
//!
//! 角（corner）と帯（band）の計算を組み合わせて、盤を一周する

use crate::{
    corner::{CornerMatrixResult, corner_transition_matrix},
    cost::{Cost, stone_number},
    display::{BoardView, Dir, print_row},
    matrix::Matrix,
    rules::{Graph, Problem, RowEdges, list_valid_rows},
    transfer::{INF, build_transition_cost_matrix},
};
use std::ops::Add;

/// 総 loss と、そのうちの帯の loss の組
///
/// フィールドの順に比べるので、総 loss が小さいほうが優先され、総 loss が同じなら帯の loss が小さいほうが優先される
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TotalAndBandLoss {
    /// 角と帯を合わせた loss
    total: i64,
    /// 帯の loss
    band: i64,
}

impl Add for TotalAndBandLoss {
    type Output = TotalAndBandLoss;

    fn add(self, other: TotalAndBandLoss) -> TotalAndBandLoss {
        TotalAndBandLoss {
            total: self.total + other.total,
            band: self.band + other.band,
        }
    }
}

/// 成分が TotalAndBandLoss の行列。None はつながらない遷移
type TotalAndBandLossMatrix = Vec<Vec<Option<TotalAndBandLoss>>>;

/// 幅 corner_size の縁を一周したときの最小コストの計算。盤は rows × cols、corner_size は角の一辺
pub fn border_cost_matrix(
    rows: usize,
    cols: usize,
    corner_size: usize,
    graph: Graph,
    problem: Problem,
    cost: Cost,
    verbose: bool,
) {
    println!(
        "border matrix: rows = {rows}, cols = {cols}, corner_size = {corner_size}, graph = {graph}, problem = {problem}, cost = {cost:?}, verbose = {verbose}"
    );

    // 帯の遷移コスト行列（左端が盤の端）
    let edges = RowEdges {
        left: true,
        right: false,
    };
    let band_rows = list_valid_rows(corner_size, graph, problem, edges);
    let band_matrix = build_transition_cost_matrix(&band_rows, graph, problem, cost, edges);
    // 角の遷移コスト行列
    let CornerMatrixResult {
        valid_rows,
        cost_matrix: corner_matrix,
        paths: corner_paths,
        rows_in_corner,
    } = corner_transition_matrix(corner_size, graph, problem, cost, verbose);

    // 左上の角から反時計回りに、帯と角を交互につないで一周する。
    // 各辺は、角の行（rows_in_corner 行）と、それに続く帯の行からなり、合わせて辺の長さ − corner_size 行
    let extra_rows_in_corner = rows_in_corner - corner_size;
    let min_side_len = corner_size * 2 + extra_rows_in_corner;
    if rows < min_side_len || cols < min_side_len {
        println!("盤が小さすぎて縁が入らない（各辺 {min_side_len} 以上が必要）");
        return;
    }
    let row_band_len = rows - min_side_len;
    let col_band_len = cols - min_side_len;
    // 角の loss は総 loss だけに入り、帯の loss は総 loss と帯の loss の両方に入る
    let corner_losses = to_total_and_band_losses(&corner_matrix, |loss| TotalAndBandLoss {
        total: loss,
        band: 0,
    });
    let band_row_losses = to_total_and_band_losses(&band_matrix, |loss| TotalAndBandLoss {
        total: loss,
        band: loss,
    });
    let mut border_losses = corner_losses.clone();
    let mut via_matrix_list = vec![];
    let mut append = |band_row_or_corner_losses: &TotalAndBandLossMatrix| {
        let (appended, via) =
            append_band_row_or_corner_to_border(&border_losses, band_row_or_corner_losses);
        border_losses = appended;
        via_matrix_list.push(via);
    };
    // 左帯 → 左下の角 → 下帯 → 右下の角 → 右帯 → 右上の角 → 上帯（左上の角は初期状態）
    let band_lens = [row_band_len, col_band_len, row_band_len, col_band_len];
    for (i, band_len) in band_lens.into_iter().enumerate() {
        for _ in 0..band_len {
            append(&band_row_losses);
        }
        if i < band_lens.len() - 1 {
            append(&corner_losses);
        }
    }

    // 初期列と最後の列が一致してるものを選択
    let (min_index, min_loss) = (0..valid_rows.len())
        .filter_map(|i| border_losses[i][i].map(|loss| (i, loss)))
        .min_by_key(|&(_, loss)| loss)
        .unwrap();
    let TotalAndBandLoss {
        total: min_cost,
        band: band_cost,
    } = min_loss;
    let min_stone = stone_number(graph, Cost::Loss, min_cost, rows, cols);

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
        let mut side_rows = corner_paths[in_index][path_list[p]].clone();
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
    println!(
        "min_cost = {min_cost}（角 {}、帯 {band_cost}）",
        min_cost - band_cost
    );
    println!("min_stone = {min_stone}");
}

/// loss の行列を TotalAndBandLoss の行列にする。loss が INF（つながらない）なら None
fn to_total_and_band_losses(
    matrix: &Matrix,
    f: impl Fn(i64) -> TotalAndBandLoss,
) -> TotalAndBandLossMatrix {
    matrix
        .iter()
        .map(|row| {
            row.iter()
                .map(|&loss| if loss >= INF { None } else { Some(f(loss)) })
                .collect()
        })
        .collect()
}

/// 縁を一周つなぐ途中で、ここまでの縁に、次の帯の 1 行か角を 1 つつなぐ
///
/// border_losses[start][current] は、縁の始まりの状態 start から今の状態 current までの (総 loss, 帯の loss)。
/// band_row_or_corner_losses[current][next] は、帯の 1 行か角で current から next へ進む (総 loss, 帯の loss)。
/// 経由する current は、総 loss が最小のものを選び、総 loss が同じなら帯の loss が小さいものを選ぶ。
/// 戻り値は、つないだ後の縁の (総 loss, 帯の loss) の表と、そのとき経由した current の表。
/// start から next へ行けないときは、どちらも None
fn append_band_row_or_corner_to_border(
    border_losses: &TotalAndBandLossMatrix,
    band_row_or_corner_losses: &TotalAndBandLossMatrix,
) -> (TotalAndBandLossMatrix, Vec<Vec<Option<usize>>>) {
    let state_count = band_row_or_corner_losses.len();
    let mut appended_border_losses = vec![vec![None; state_count]; border_losses.len()];
    let mut via = vec![vec![None; state_count]; border_losses.len()];
    for (start, border_losses_from_start) in border_losses.iter().enumerate() {
        for (current, &border_loss_to_current) in border_losses_from_start.iter().enumerate() {
            // 縁が current まで来られないなら飛ばす
            let Some(border_loss_to_current) = border_loss_to_current else {
                continue;
            };
            for (next, &band_row_or_corner_loss) in
                band_row_or_corner_losses[current].iter().enumerate()
            {
                // 帯の 1 行か角で current から next へ進めないなら飛ばす
                let Some(band_row_or_corner_loss) = band_row_or_corner_loss else {
                    continue;
                };
                let border_loss_to_next = border_loss_to_current + band_row_or_corner_loss;
                let min_border_loss_to_next = &mut appended_border_losses[start][next];
                if min_border_loss_to_next.is_none_or(|min| border_loss_to_next < min) {
                    *min_border_loss_to_next = Some(border_loss_to_next);
                    via[start][next] = Some(current);
                }
            }
        }
    }
    (appended_border_losses, via)
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
