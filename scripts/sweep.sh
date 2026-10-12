#!/usr/bin/env bash
# n × n の盤の最小コストを、n を変えながら順に計算して「n: 最小コスト」の形で表示する
#
# 使い方: bash scripts/sweep.sh [dom|fair] [最小の n] [最大の n] [board に渡す追加の引数...]
#   例: bash scripts/sweep.sh fair 2 15
#       bash scripts/sweep.sh dom 2 8 --method dfs

set -euo pipefail

problem="${1:-fair}"
from="${2:-2}"
to="${3:-15}"
shift $(($# < 3 ? $# : 3))

# どこから呼んでもリポジトリの直下で動かす
cd "$(dirname "$0")/.."

cargo build --release --quiet

for n in $(seq "$from" "$to"); do
    start=$EPOCHREALTIME
    cost=$(./target/release/border-loss board -n "$n" -m "$n" --problem "$problem" "$@" |
        sed -n 's/^最小コスト: //p')
    end=$EPOCHREALTIME
    seconds=$(awk -v s="$start" -v e="$end" 'BEGIN { printf "%.3f", e - s }')
    echo "$n: $cost (${seconds}s)"
done
