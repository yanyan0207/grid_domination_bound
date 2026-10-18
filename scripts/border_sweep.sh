#!/usr/bin/env bash
# 縁（border）の計算で求めた最小の石の数を、盤の大きさを変えながら順に計算して表示する
#
# 使い方: bash scripts/border_sweep.sh [dom|fair] [角の一辺] [最小の n] [最大の n] [最小の m] [最大の m] [grid|king]
#   m の範囲を省略すると n × n の盤を順に計算して 1 行ずつ表示する
#   m の範囲を指定すると n × m の盤を計算して表（行が n、列が m）にする。
#   盤を回すと n と m が入れ替わるだけなので n ≤ m だけ計算する
#   n, m が角の一辺の 2 倍（king は 2 倍 + 1）未満のときは縁が入らないので計算しない（表では「-」）
#   m の範囲を指定せずに king にするときは、m の範囲に "" を 2 つ渡す
#   例: bash scripts/border_sweep.sh dom 4 8 20
#       bash scripts/border_sweep.sh dom 4 8 12 8 20
#       bash scripts/border_sweep.sh fair 4 9 20 "" "" king

set -euo pipefail

problem="${1:-dom}"
corner_size="${2:-4}"
from="${3:-$((corner_size * 2))}"
to="${4:-20}"
m_from="${5:-}"
m_to="${6:-}"
graph="${7:-grid}"

# どこから呼んでもリポジトリの直下で動かす
cd "$(dirname "$0")/.."

cargo build --release --quiet

# 縁が入る最小の盤。king の角は 1 行多い
if [[ "$graph" == king ]]; then
    min_size=$((corner_size * 2 + 1))
else
    min_size=$((corner_size * 2))
fi

# n × m の盤の最小の石の数
min_stone() {
    ./target/release/border-loss border -n "$1" -m "$2" -c "$corner_size" --graph "$graph" --problem "$problem" |
        sed -n 's/^min_stone = //p'
}

if [[ -z "$m_from" ]]; then
    # n × n
    for n in $(seq "$from" "$to"); do
        if ((n < min_size)); then
            continue
        fi
        start=$EPOCHREALTIME
        stones=$(min_stone "$n" "$n")
        end=$EPOCHREALTIME
        seconds=$(awk -v s="$start" -v e="$end" 'BEGIN { printf "%.3f", e - s }')
        echo "$n: stones = $stones (${seconds}s)"
    done
else
    # n × m の表
    m_to="${m_to:-$m_from}"
    printf '%4s' 'n\m'
    for m in $(seq "$m_from" "$m_to"); do
        printf '%6d' "$m"
    done
    echo
    for n in $(seq "$from" "$to"); do
        printf '%4d' "$n"
        for m in $(seq "$m_from" "$m_to"); do
            if ((n < min_size || m < min_size || m < n)); then
                printf '%6s' '-'
            else
                printf '%6d' "$(min_stone "$n" "$m")"
            fi
        done
        echo
    done
fi
