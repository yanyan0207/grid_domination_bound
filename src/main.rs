//! コマンドラインの入口。引数を解析して、各サブコマンドの処理に振り分ける

mod board;
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
        /// 盤の行数（列の状態の高さ）
        #[arg(short = 'n', long = "lines", default_value_t = 6)]
        n: usize,
        /// 盤の列数（遷移を進める回数）
        #[arg(short = 'm', long = "columns", default_value_t = 6)]
        m: usize,
        /// 問題の種類
        #[arg(long, value_enum, default_value_t = ProblemArg::Dom)]
        problem: ProblemArg,
    },
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
        Command::Board { n, m, problem } => board::board(n, m, problem.into()),
    }
}
