#!/usr/bin/env bash
# 幅 n の盤（左右と上が盤の端）を m 方向に伸ばす DP の周期を、n ごとに CSV で標準出力に出す
#
# 使い方: bash scripts/period_sweep_csv.sh [--problem dom|fair] [--cost stones|loss]
#                                           [--n-from 最小の n] [--n-to 最大の n] [--max-rows 進める行数の上限] > 出力.csv
#   既定は --problem fair --cost stones --n-from 2 --n-to 9 --max-rows 100
#   例: bash scripts/period_sweep_csv.sh --problem fair --n-from 2 --n-to 9 > results/period_fair_small.csv
#
# 出力の列
#   n      盤の幅
#   r0     周期が始まるまでに、上端の行から進めた行数
#   m0     周期が始まる盤の行数（= r0 + 1）。m ≥ m0 で、m を k 増やすとコストがちょうど p 増える
#   k, p   周期と、1 周期あたりのコストの増え方（--cost stones なら石の数）
#   周期が見つからなければ r0 から p までが空

set -euo pipefail

problem=fair
cost=stones
n_from=2
n_to=9
max_rows=100

while [[ $# -gt 0 ]]; do
    case "$1" in
    --problem) problem="$2" ;;
    --cost) cost="$2" ;;
    --n-from) n_from="$2" ;;
    --n-to) n_to="$2" ;;
    --max-rows) max_rows="$2" ;;
    *)
        echo "知らない引数: $1" >&2
        exit 1
        ;;
    esac
    shift 2
done

# どこから呼んでもリポジトリの直下で動かす
cd "$(dirname "$0")/.."

cargo build --release --quiet

echo "n,r0,m0,k,p"
for n in $(seq "$n_from" "$n_to"); do
    output=$(./target/release/border-loss period -w "$n" --problem "$problem" --cost "$cost" \
        --left edge --right edge --top edge --max-rows "$max_rows")
    # 「r0 = 28, k = 3, p = 6」の行から数を取り出す
    read -r r0 k p < <(sed -n 's/^r0 = \([0-9]*\), k = \([0-9]*\), p = \(-\?[0-9]*\)$/\1 \2 \3/p' <<<"$output") || true
    if [[ -n "${r0:-}" ]]; then
        echo "$n,$r0,$((r0 + 1)),$k,$p"
    else
        echo "$n,,,,"
    fi
    unset r0 k p
done
