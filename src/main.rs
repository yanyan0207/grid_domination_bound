//! コマンドラインの入口。引数を解析して、各サブコマンドの処理に振り分ける

mod board;
mod border;
mod corner;
mod cost;
mod display;
mod dp;
mod matrix;
mod rules;
mod transfer;

use clap::{Parser, Subcommand, ValueEnum};
use rules::Problem;

#[derive(Parser)]
#[command(name = "border-loss")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 盤全体の最小コストの計算
    Board {
        /// 盤の行数（上から 1 行ずつ積む回数）
        #[arg(short = 'n', long = "lines", default_value_t = 6)]
        n: usize,
        /// 盤の列数（1 行の状態の幅）
        #[arg(short = 'm', long = "columns", default_value_t = 6)]
        m: usize,
        /// 問題の種類
        #[arg(long, value_enum, default_value_t = ProblemArg::Dom)]
        problem: ProblemArg,
        /// 盤面の表示で Ok と NeedOne も区別する
        #[arg(short, long)]
        verbose: bool,
        /// コストの種類
        #[arg(long, value_enum, default_value_t = CostArg::Stones)]
        cost: CostArg,
    },
    /// 縁（角と帯）の計算
    Border {
        /// 縁の長さ（上から 1 行ずつ積む回数）
        #[arg(short = 'n', long = "lines", default_value_t = 6)]
        n: usize,
        /// 縁の幅（1 行の状態の幅）
        #[arg(short = 'm', long = "columns", default_value_t = 6)]
        m: usize,
        /// 角の一辺の長さ
        #[arg(short = 'c', long = "corner-size", default_value_t = 6)]
        corner_size: usize,
        /// 問題の種類
        #[arg(long, value_enum, default_value_t = ProblemArg::Dom)]
        problem: ProblemArg,
        /// 詳しい表示
        #[arg(short, long)]
        verbose: bool,
    },
    /// 角の計算
    Corner {
        /// 角の一辺の長さ
        #[arg(short = 'c', long = "corner-size", default_value_t = 6)]
        corner_size: usize,
        /// 問題の種類
        #[arg(long, value_enum, default_value_t = ProblemArg::Dom)]
        problem: ProblemArg,
        /// 詳しい表示
        #[arg(short, long)]
        verbose: bool,
    },
}

/// --cost の選択肢。cost.rs を clap に依存させないため、ここで対応づける
#[derive(Clone, Copy, ValueEnum)]
enum CostArg {
    /// 石の数
    Stones,
    /// loss
    Loss,
}

impl From<CostArg> for cost::Cost {
    fn from(arg: CostArg) -> Self {
        match arg {
            CostArg::Stones => cost::Cost::Stones,
            CostArg::Loss => cost::Cost::Loss,
        }
    }
}

/// --problem の選択肢。rules.rs を clap に依存させないため、ここで対応づける
#[derive(Clone, Copy, ValueEnum)]
enum ProblemArg {
    /// 2-domination
    Dom,
    /// 2-fair
    Fair,
}

impl From<ProblemArg> for Problem {
    fn from(arg: ProblemArg) -> Self {
        match arg {
            ProblemArg::Dom => Problem::Dom2,
            ProblemArg::Fair => Problem::Fair2,
        }
    }
}

fn main() {
    match Cli::parse().command {
        Command::Board {
            n,
            m,
            problem,
            verbose,
            cost,
        } => {
            board::board(n, m, problem.into(), verbose, cost.into());
        }
        Command::Border {
            n,
            m,
            corner_size,
            problem,
            verbose,
        } => {
            border::border_cost_matrix(n, m, corner_size, problem.into(), cost::Cost::Loss, verbose)
        }
        Command::Corner {
            corner_size,
            problem,
            verbose,
        } => {
            corner::corner(corner_size, problem.into(), cost::Cost::Loss, verbose);
        }
    }
}
