#!/usr/bin/env bash
# n × n の盤の縁（border）の最小の石の数を、角の一辺ごとに列にした CSV を標準出力に出す
#
# 使い方: bash scripts/border_sweep_csv.sh [dom|fair] [最小の角の一辺] [最大の角の一辺] [最小の n] [最大の n] [grid|king] > 出力.csv
#   n が角の一辺の 2 倍（king は 2 倍 + 1）未満のときは縁が入らないので空欄
#   例: bash scripts/border_sweep_csv.sh dom 2 6 8 30 > results/border_dom.csv
#       bash scripts/border_sweep_csv.sh fair 2 6 8 30 king > results/border_king_fair.csv

set -euo pipefail

problem="${1:-dom}"
c_from="${2:-2}"
c_to="${3:-6}"
from="${4:-8}"
to="${5:-30}"
graph="${6:-grid}"

# king の角は 1 行多いので、縁が入る最小の盤が 1 大きい
if [[ "$graph" == king ]]; then
    extra_rows_in_corner=1
else
    extra_rows_in_corner=0
fi

# どこから呼んでもリポジトリの直下で動かす
cd "$(dirname "$0")/.."

cargo build --release --quiet

header="n"
for c in $(seq "$c_from" "$c_to"); do
    header+=",c=$c"
done
echo "$header"

for n in $(seq "$from" "$to"); do
    line="$n"
    for c in $(seq "$c_from" "$c_to"); do
        line+=","
        if ((n >= c * 2 + extra_rows_in_corner)); then
            line+=$(./target/release/border-loss border -n "$n" -m "$n" -c "$c" --graph "$graph" --problem "$problem" |
                sed -n 's/^min_stone = //p')
        fi
    done
    echo "$line"
done
