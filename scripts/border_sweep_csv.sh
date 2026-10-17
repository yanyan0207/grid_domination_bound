#!/usr/bin/env bash
# n × n の盤の縁（border）の最小の石の数を、角の一辺ごとに列にした CSV を標準出力に出す
#
# 使い方: bash scripts/border_sweep_csv.sh [dom|fair] [最小の角の一辺] [最大の角の一辺] [最小の n] [最大の n] > 出力.csv
#   n が角の一辺の 2 倍未満のときは縁が入らないので空欄
#   例: bash scripts/border_sweep_csv.sh dom 2 6 8 30 > results/border_dom.csv

set -euo pipefail

problem="${1:-dom}"
c_from="${2:-2}"
c_to="${3:-6}"
from="${4:-8}"
to="${5:-30}"

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
        if ((n >= c * 2)); then
            line+=$(./target/release/border-loss border -n "$n" -m "$n" -c "$c" --problem "$problem" |
                sed -n 's/^min_stone = //p')
        fi
    done
    echo "$line"
done
