#!/usr/bin/env bash
# 2-domination の値と、探索で使った値を全部計算して results/dom2/ に保存する（格子グラフ）。
# Rao と Talon の結果の再現と検証に使う
#
# 使い方: bash scripts/collect_dom2.sh
#   範囲は下の「範囲」の変数で変える
#
# 保存するもの
#   検証に使うもの
#     period_band_c6.txt    幅 6 の帯の周期（縁の帯: 左端は盤の端、右端と始まりの行は開いている）
#     border_rect_c6.txt    幅 6 の縁の下界の表（13 ≤ n ≤ m ≤ rect_to）
#     board_small.csv       小さい行数の厳密な値（2 ≤ n ≤ 12、n ≤ m ≤ small_m_to）
#     period_small.csv      小さい行数の m 方向の周期（2 ≤ n ≤ 12）
#   探索に使ったもの（幅をいくつにすれば十分かの比較）
#     period_band_all.txt   幅 2〜6 の帯の周期
#     border_square.csv     n × n の縁の下界を、角の一辺 2〜6 ごとに並べた表（n ≤ square_to）
#
# 範囲の決め方
#   縁の帯の長さは n − 2c − 1 なので、帯の周期が r0 から始まるなら、n ≥ 2c + 1 + r0 で周期どおりに増える。
#   帰納法の出発点は 2c + 1 + r0 からの 3 つなので、表は少なくともそこまで計算する（幅 6 は r0 = 20 なので 35。Rao と Talon と同じ）。
#   幅 6 の縁は n ≥ 13 で入るので、小さい行数は n ≤ 12 を厳密に計算する

set -euo pipefail

# 範囲
rect_to=40     # 幅 6 の縁の表の n, m の上限（検証には 35 まで必要）
small_m_to=50  # 小さい行数の厳密な値の m の上限
square_to=50   # n × n の比較表の n の上限

# どこから呼んでもリポジトリの直下で動かす
cd "$(dirname "$0")/.."

cargo build --release --quiet

out=results/dom2
mkdir -p "$out"

echo "幅 2〜6 の帯の周期" >&2
: >"$out/period_band_all.txt"
for width in 2 3 4 5 6; do
    ./target/release/border-loss period -w "$width" --problem dom --cost loss \
        --left edge --right open --top open >"$out/period_band_c$width.txt"
    cat "$out/period_band_c$width.txt" >>"$out/period_band_all.txt"
done
r0_c6=$(sed -n 's/^r0 = \([0-9]*\), k = .*/\1/p' "$out/period_band_c6.txt")
if [[ -z "$r0_c6" ]]; then
    echo "幅 6 の帯の周期が見つからなかった（$out/period_band_c6.txt を確認）" >&2
    exit 1
fi
echo "  幅 6: r0 = $r0_c6" >&2

n_to_c6=$((2 * 6 + 1 + r0_c6 + 2))
if ((n_to_c6 < rect_to)); then
    n_to_c6=$rect_to
fi
echo "幅 6 の縁の下界（13 ≤ n ≤ m ≤ $n_to_c6）" >&2
bash scripts/border_sweep.sh dom 6 13 "$n_to_c6" 13 "$n_to_c6" >"$out/border_rect_c6.txt"

echo "n × n の縁の下界の比較（角の一辺 2〜6、n ≤ $square_to）" >&2
bash scripts/border_sweep_csv.sh dom 2 6 5 "$square_to" >"$out/border_square.csv"

echo "小さい行数の厳密な値（2 ≤ n ≤ 12、n ≤ m ≤ $small_m_to）" >&2
bash scripts/board_sweep_csv.sh dom 2 12 "$small_m_to" >"$out/board_small.csv"

echo "小さい行数の m 方向の周期（2 ≤ n ≤ 12）" >&2
bash scripts/period_sweep_csv.sh --problem dom --n-from 2 --n-to 12 >"$out/period_small.csv"

echo "完了: $out" >&2
