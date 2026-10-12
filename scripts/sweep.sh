#!/usr/bin/env bash
# n × n の盤の最小コストと石の数を、n を変えながら順に計算して表示する
#
# 使い方: bash scripts/sweep.sh [dom|fair] [最小の n] [最大の n] [stones|loss] [board に渡す追加の引数...]
#   例: bash scripts/sweep.sh fair 2 15
#       bash scripts/sweep.sh dom 2 8 loss
#       bash scripts/sweep.sh dom 2 8 stones --method dfs

set -euo pipefail

problem="${1:-fair}"
from="${2:-2}"
to="${3:-15}"
cost="${4:-stones}"
shift $(($# < 4 ? $# : 4))

# どこから呼んでもリポジトリの直下で動かす
cd "$(dirname "$0")/.."

cargo build --release --quiet

for n in $(seq "$from" "$to"); do
    start=$EPOCHREALTIME
    output=$(./target/release/border-loss board -n "$n" -m "$n" --problem "$problem" --cost "$cost" "$@")
    end=$EPOCHREALTIME
    min_cost=$(sed -n 's/^最小コスト: //p' <<<"$output")
    stones=$(sed -n 's/^石の数: //p' <<<"$output")
    seconds=$(awk -v s="$start" -v e="$end" 'BEGIN { printf "%.3f", e - s }')
    echo "$n: cost = $min_cost, stones = $stones (${seconds}s)"
done
