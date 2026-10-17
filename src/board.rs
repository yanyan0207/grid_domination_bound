//! 盤全体（n × m）の最小コストの計算

use crate::{
    cost::{Cost, stone_number},
    dfs::dfs,
    display::print_row_with_cost,
    dp::{calc_dp, init_dp, print_dp_result},
    rules::{Edges, Problem, list_valid_rows},
};

/// 最小コストを求める方式
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// 転送行列（列ごとの DP）
    Dp,
    /// 全探索
    Dfs,
}

/// n × m の盤全体の最小コストの計算。(最小コスト, 石の数) を返す
pub fn board(
    rows: usize,
    cols: usize,
    problem: Problem,
    verbose: bool,
    method: Method,
    cost: Cost,
) -> (i64, i64) {
    let (min_cost, stone_num) = match method {
        Method::Dp => {
            let dp_config = init_dp(rows, cols, problem, cost, Edges::ALL, verbose);
            let result = calc_dp(&dp_config, None, verbose);
            print_dp_result(&dp_config, &result);
            (result.min_cost, result.stone_num)
        }
        Method::Dfs => {
            if cost == Cost::Loss {
                unimplemented!("DFS は loss に対応していない");
            }
            // 有効な行のリストを取得(四方とも端)
            let edges = Edges::ALL;
            let row_edges = edges.row();
            let valid_rows = list_valid_rows(cols, row_edges);
            // 1 行目のコスト
            let first_row_costs: Vec<i64> = valid_rows
                .iter()
                .map(|row| cost.first_row(row, &edges))
                .collect();

            // コストは 1 行目のコストと、隣り合う行の遷移コストの合計
            let (min_cost, min_pattern) = dfs(&valid_rows, rows, problem, |index_list| {
                let transition_costs: i64 = index_list
                    .windows(2)
                    .map(|pair| {
                        cost.transition(&valid_rows[pair[0]], &valid_rows[pair[1]], row_edges)
                    })
                    .sum();
                first_row_costs[index_list[0]] + transition_costs
            });
            if verbose {
                println!("最小コスト: {min_cost}");
            }
            // 経路の各行で増えたコストと、その行までの累計
            let mut total = 0;
            for (i, &index) in min_pattern.iter().enumerate() {
                let step = if i == 0 {
                    first_row_costs[index]
                } else {
                    cost.transition(
                        &valid_rows[min_pattern[i - 1]],
                        &valid_rows[index],
                        row_edges,
                    )
                };
                total += step;
                print_row_with_cost(&valid_rows[index], verbose, Some((step, total)));
            }
            (min_cost, stone_number(cost, min_cost, rows, cols))
        }
    };
    if verbose {
        println!("石の数: {stone_num}");
    }
    (min_cost, stone_num)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// arXiv:1810.12896 の著者のツール（Source/release.exe）の出力。(行数 n, 列数 m, 最小の石の数)。幅 1 は扱わない
    const DOM2_EXPECTED: [(usize, usize, i64); 12] = [
        (2, 2, 2),
        (2, 3, 3),
        (2, 4, 4),
        (2, 5, 5),
        (2, 6, 6),
        (3, 3, 4),
        (3, 4, 6),
        (3, 5, 7),
        (3, 6, 8),
        (4, 4, 8),
        (4, 5, 10),
        (4, 6, 12),
    ];

    /// OEIS A344719（n × n の 2-fair の最小の石の数）の a(2)〜a(6)。幅 1 は扱わない
    const FAIR2_EXPECTED: [(usize, i64); 5] = [(2, 2), (3, 5), (4, 8), (5, 11), (6, 17)];

    /// 2-domination の石の数が、著者のツールの値と一致するか
    fn check_dom2(method: Method, cost: Cost) {
        for (n, m, stones) in DOM2_EXPECTED {
            assert_eq!(
                board(n, m, Problem::Dom2, false, method, cost).1,
                stones,
                "n = {n}, m = {m}"
            );
        }
    }

    /// 2-fair の石の数が、OEIS の値と一致するか
    fn check_fair2(method: Method, cost: Cost) {
        for (n, stones) in FAIR2_EXPECTED {
            assert_eq!(
                board(n, n, Problem::Fair2, false, method, cost).1,
                stones,
                "n = {n}"
            );
        }
    }

    #[test]
    fn dp_stones_dom2() {
        check_dom2(Method::Dp, Cost::Stones);
    }

    #[test]
    fn dp_stones_fair2() {
        check_fair2(Method::Dp, Cost::Stones);
    }

    #[test]
    fn dfs_stones_dom2() {
        check_dom2(Method::Dfs, Cost::Stones);
    }

    #[test]
    fn dfs_stones_fair2() {
        check_fair2(Method::Dfs, Cost::Stones);
    }

    #[test]
    fn dp_loss_dom2() {
        check_dom2(Method::Dp, Cost::Loss);
    }

    #[test]
    fn dp_loss_fair2() {
        check_fair2(Method::Dp, Cost::Loss);
    }
}
