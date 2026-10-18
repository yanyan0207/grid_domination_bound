//! 盤全体（n × m）の最小コストの計算

use crate::{
    cost::Cost,
    dp::{calc_dp, init_dp, print_dp_result},
    rules::{Edges, Problem},
};

/// n × m の盤全体の最小コストの計算。(最小コスト, 石の数) を返す
pub fn board(rows: usize, cols: usize, problem: Problem, verbose: bool, cost: Cost) -> (i64, i64) {
    let dp_config = init_dp(rows, cols, problem, cost, Edges::ALL, verbose);
    let result = calc_dp(&dp_config, None, verbose);
    print_dp_result(&dp_config, &result);
    let (min_cost, stone_num) = (result.min_cost, result.stone_num);
    // 盤全体の置き方では 6|D| = 2nm + loss がちょうど成り立つので、石の数から loss を求める
    // （--cost stones のときは DP が loss を計算しないため）
    let loss = 6 * stone_num - 2 * (rows * cols) as i64;
    println!("石の数: {stone_num}");
    println!("loss: {loss}");
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
    fn check_dom2(cost: Cost) {
        for (n, m, stones) in DOM2_EXPECTED {
            assert_eq!(
                board(n, m, Problem::Dom2, false, cost).1,
                stones,
                "n = {n}, m = {m}"
            );
        }
    }

    /// 2-fair の石の数が、OEIS の値と一致するか
    fn check_fair2(cost: Cost) {
        for (n, stones) in FAIR2_EXPECTED {
            assert_eq!(
                board(n, n, Problem::Fair2, false, cost).1,
                stones,
                "n = {n}"
            );
        }
    }

    #[test]
    fn dp_stones_dom2() {
        check_dom2(Cost::Stones);
    }

    #[test]
    fn dp_stones_fair2() {
        check_fair2(Cost::Stones);
    }

    #[test]
    fn dp_loss_dom2() {
        check_dom2(Cost::Loss);
    }

    #[test]
    fn dp_loss_fair2() {
        check_fair2(Cost::Loss);
    }
}
