#!/usr/bin/env bash
# 2-fair の論文に使う値と、探索で使った値を全部計算して results/fair2/ に保存する（格子グラフ）
#
# 使い方: bash scripts/collect_fair2.sh
#   範囲は下の「範囲」の変数で変える
#
# 保存するもの
#   証明に使うもの
#     period_band_c5.txt    幅 5 の帯の周期（縁の帯: 左端は盤の端、右端と始まりの行は開いている）
#     border_rect_c5.txt    幅 5 の縁の下界の表（11 ≤ n ≤ m ≤ rect_to）
#     board_small.csv       小さい行数の厳密な値（2 ≤ n ≤ 10、n ≤ m ≤ small_m_to）
#     period_small.csv      小さい行数の m 方向の周期（2 ≤ n ≤ 10）
#   探索に使ったもの（幅をいくつにすれば十分かの比較）
#     period_band_all.txt   幅 2〜6 の帯の周期
#     border_square.csv     n × n の縁の下界を、角の一辺 2〜6 ごとに並べた表（n ≤ square_to）
#
# 範囲の決め方
#   縁の帯の長さは n − 2c − 1 なので、帯の周期が r0 から始まるなら、n ≥ 2c + 1 + r0 で周期どおりに増える。
#   帰納法の出発点は 2c + 1 + r0 からの 3 つなので、表は少なくともそこまで計算する（幅 5 は r0 = 28 なので 41）。
#   幅 5 の縁は n ≥ 11 で入るので、小さい行数は n ≤ 10 を厳密に計算する

set -euo pipefail

# 範囲
rect_to=50     # 幅 5 の縁の表の n, m の上限（証明には 41 まで必要）
small_m_to=50  # 小さい行数の厳密な値の m の上限（n ≤ 9 では 22 まで必要。n = 10 は周期を見て確かめる）
square_to=50   # n × n の比較表の n の上限

# どこから呼んでもリポジトリの直下で動かす
cd "$(dirname "$0")/.."

cargo build --release --quiet

out=results/fair2
mkdir -p "$out"

# 縁の帯の周期を計算して file に保存する
band_period() {
    local width="$1" file="$2"
    ./target/release/border-loss period -w "$width" --problem fair --cost loss \
        --left edge --right open --top open >"$file"
}

# 保存した周期のファイルから r0 を取り出す
r0_of() {
    sed -n 's/^r0 = \([0-9]*\), k = .*/\1/p' "$1"
}

echo "幅 2〜6 の帯の周期" >&2
: >"$out/period_band_all.txt"
for width in 2 3 4 5 6; do
    band_period "$width" "$out/period_band_c$width.txt"
    cat "$out/period_band_c$width.txt" >>"$out/period_band_all.txt"
done
# 証明に使うのは幅 5 だけ。ほかの幅のファイルは比較用に消さずに残す
r0_c5=$(r0_of "$out/period_band_c5.txt")
if [[ -z "$r0_c5" ]]; then
    echo "幅 5 の帯の周期が見つからなかった（$out/period_band_c5.txt を確認）" >&2
    exit 1
fi
echo "  幅 5: r0 = $r0_c5" >&2

echo "幅 5 の縁の下界（11 ≤ n ≤ m ≤ $rect_to）" >&2
bash scripts/border_sweep.sh fair 5 11 "$rect_to" 11 "$rect_to" >"$out/border_rect_c5.txt"

echo "n × n の縁の下界の比較（角の一辺 2〜6、n ≤ $square_to）" >&2
bash scripts/border_sweep_csv.sh fair 2 6 5 "$square_to" >"$out/border_square.csv"

echo "小さい行数の厳密な値（2 ≤ n ≤ 10、n ≤ m ≤ $small_m_to）" >&2
bash scripts/board_sweep_csv.sh fair 2 10 "$small_m_to" >"$out/board_small.csv"

echo "小さい行数の m 方向の周期（2 ≤ n ≤ 10）" >&2
bash scripts/period_sweep_csv.sh --problem fair --n-from 2 --n-to 10 >"$out/period_small.csv"

echo "完了: $out" >&2
