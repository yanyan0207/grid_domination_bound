#!/usr/bin/env bash
# 盤全体の厳密な計算（board）で求めた n × m の最小の石の数を、n ≤ m の組ごとに CSV で標準出力に出す
#
# 使い方: bash scripts/board_sweep_csv.sh [dom|fair] [最小の n] [最大の n] [最大の m] [grid|king] > 出力.csv
#   短い辺 n を 1 行の幅、長い辺 m を行数にして計算する（盤を回しても値は同じで、幅が小さいほど速い）
#   board は幅 1 を扱わないので、n は 2 以上
#   例: bash scripts/board_sweep_csv.sh fair 2 9 40 > results/board_fair_small.csv
#       bash scripts/board_sweep_csv.sh fair 2 9 40 king > results/board_king_fair_small.csv

set -euo pipefail

problem="${1:-fair}"
from="${2:-2}"
to="${3:-9}"
m_to="${4:-40}"
graph="${5:-grid}"

# どこから呼んでもリポジトリの直下で動かす
cd "$(dirname "$0")/.."

cargo build --release --quiet

echo "n,m,stones,loss"
for n in $(seq "$from" "$to"); do
    for m in $(seq "$n" "$m_to"); do
        output=$(./target/release/border-loss board -n "$m" -m "$n" --graph "$graph" --problem "$problem")
        stones=$(sed -n 's/^石の数: //p' <<<"$output")
        loss=$(sed -n 's/^loss: //p' <<<"$output")
        echo "$n,$m,$stones,$loss"
    done
done
