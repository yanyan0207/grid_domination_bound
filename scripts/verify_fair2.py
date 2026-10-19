"""2-fair の論文の計算による証明を、results/fair2/ の値と照合する

使い方: python scripts/verify_fair2.py
    先に bash scripts/collect_fair2.sh で results/fair2/ を作っておく

確かめること（論文の該当箇所）
    1. 小さい行数（3 節、定理と周期の表）
       - period_small.csv の m0, k, p が、論文の表と同じ
       - board_small.csv の値が、定理の式と同じ。とくに、証明に必要な n ≤ m < m0 + k の値（計 95 個）がそろっている
    2. 大きい行数（4 節）
       - period_band_c5.txt が r0 = 28, k = 3, p = 6（B^31 = B^28 + 6）
       - border_rect_c5.txt の 11 ≤ n ≤ m ≤ 41 の 496 個がそろっていて、どれも縁の下界が F(n, m) に等しい
全部通れば終了コード 0、どれかが違えば 1 で終わる
"""

import csv
import re
import sys
from pathlib import Path

RESULTS = Path(__file__).resolve().parent.parent / "results" / "fair2"


def ceil_div(numerator: int, denominator: int) -> int:
    return -(-numerator // denominator)


def lower_bound_formula(n: int, m: int) -> int:
    """論文の F(n, m) = ⌈(n+2)(m+2)/3⌉ − 6 + [n ≢ m (mod 3)]"""
    return ceil_div((n + 2) * (m + 2), 3) - 6 + (1 if n % 3 != m % 3 else 0)


def small_height_formula(n: int, m: int) -> int:
    """論文の定理（小さい行数）の、幅 n の盤の 2-fair 支配数"""
    if n == 2:
        return 2 * ceil_div(m, 2)
    if n == 3:
        return (7 * m + 6) // 5 + (1 if m % 5 == 1 else 0)
    if n == 4:
        return (16 * m + 14) // 9
    if n == 5:
        return (11 * m + 4) // 5 + (1 if m % 5 == 4 else 0)
    if n == 6:
        return (34 * m + 24) // 13
    if n == 7:
        return 3 * m + (1 if m in (8, 9, 11, 12) else 0)
    if n == 8:
        return (10 * m + 6) // 3
    if n == 9:
        return (11 * m + 8) // 3
    if n == 10:
        return 4 * m + 2 + (1 if m % 3 != 1 else 0)
    raise ValueError(f"n = {n} の式は論文にない")


# 論文の周期の表: n -> (m0, k, p)。m ≥ m0 で V_{m+k} = V_m + p
PAPER_SMALL_PERIODS = {
    2: (3, 2, 2),
    3: (5, 5, 7),
    4: (10, 9, 16),
    5: (9, 5, 11),
    6: (10, 13, 34),
    7: (19, 1, 3),
    8: (14, 3, 10),
    9: (18, 3, 11),
    10: (17, 3, 12),
}

# 論文の補題（帯の行列の周期）: 幅 5 で B^(r0 + k) = B^r0 + p
PAPER_BAND_PERIOD = (28, 3, 6)

# 論文の命題（大きい行数）の有限個の確認の範囲
LARGE_N_FROM = 11
LARGE_N_TO = 41


class Report:
    """確かめた結果を 1 行ずつ表示し、失敗があったかを覚えておく"""

    def __init__(self) -> None:
        self.has_failure = False

    def check(self, passed: bool, description: str, details: str = "") -> None:
        mark = "OK" if passed else "NG"
        print(f"[{mark}] {description}")
        if not passed:
            self.has_failure = True
            if details:
                print(f"     {details}")


def read_small_periods() -> dict[int, tuple[int, int, int]]:
    periods = {}
    with open(RESULTS / "period_small.csv", encoding="utf-8") as file:
        for row in csv.DictReader(file):
            if row["k"]:
                periods[int(row["n"])] = (int(row["m0"]), int(row["k"]), int(row["p"]))
    return periods


def read_small_values() -> dict[tuple[int, int], int]:
    values = {}
    with open(RESULTS / "board_small.csv", encoding="utf-8") as file:
        for row in csv.DictReader(file):
            if row["stones"]:
                values[(int(row["n"]), int(row["m"]))] = int(row["stones"])
    return values


def read_band_period() -> tuple[int, int, int] | None:
    text = (RESULTS / "period_band_c5.txt").read_text(encoding="utf-8")
    found = re.search(r"^r0 = (\d+), k = (\d+), p = (-?\d+)$", text, re.MULTILINE)
    if found is None:
        return None
    return int(found.group(1)), int(found.group(2)), int(found.group(3))


def read_border_table() -> dict[tuple[int, int], int]:
    """border_sweep.sh の表（1 行目が m、各行の先頭が n、計算しないところは「-」）"""
    lines = (RESULTS / "border_rect_c5.txt").read_text(encoding="utf-8").splitlines()
    column_ms = [int(text) for text in lines[0].split()[1:]]
    values = {}
    for line in lines[1:]:
        cells = line.split()
        if not cells:
            continue
        n = int(cells[0])
        for m, text in zip(column_ms, cells[1:]):
            if text != "-":
                values[(n, m)] = int(text)
    return values


def verify_small_heights(report: Report) -> None:
    computed_periods = read_small_periods()
    for n, paper_period in PAPER_SMALL_PERIODS.items():
        computed = computed_periods.get(n)
        report.check(
            computed == paper_period,
            f"小さい行数 n = {n}: 周期 (m0, k, p) が論文の表と同じ",
            f"計算 {computed}、論文 {paper_period}",
        )

    values = read_small_values()
    needed_count = 0
    for n, (m0, k, _) in PAPER_SMALL_PERIODS.items():
        # 証明に必要なのは、周期が効き始めるまでの n ≤ m < m0 + k
        needed_ms = range(n, m0 + k)
        missing = [m for m in needed_ms if (n, m) not in values]
        needed_count += len(needed_ms)
        report.check(
            not missing,
            f"小さい行数 n = {n}: 証明に必要な m = {n}〜{m0 + k - 1} の値がそろっている",
            f"ない m: {missing}",
        )
        mismatches = [
            (m, stones, small_height_formula(n, m))
            for (value_n, m), stones in sorted(values.items())
            if value_n == n and stones != small_height_formula(n, m)
        ]
        report.check(
            not mismatches,
            f"小さい行数 n = {n}: 計算したすべての値（m ≤ {max(m for value_n, m in values if value_n == n)}）が定理の式と同じ",
            f"(m, 計算, 式): {mismatches[:5]}",
        )
    report.check(needed_count == 95, f"小さい行数: 証明に必要な値の数が論文と同じ 95 個（数えると {needed_count} 個）")

    for n in (8, 9, 10):
        mismatches = [
            m
            for (value_n, m) in values
            if value_n == n and small_height_formula(n, m) != lower_bound_formula(n, m)
        ]
        report.check(
            not mismatches,
            f"小さい行数 n = {n}: 定理の式が F(n, m) に等しい（主定理の n = {n}）",
            f"違う m: {sorted(mismatches)[:5]}",
        )


def verify_large_heights(report: Report) -> None:
    band_period = read_band_period()
    report.check(
        band_period == PAPER_BAND_PERIOD,
        "大きい行数: 幅 5 の帯の行列が B^31 = B^28 + 6（r0 = 28, k = 3, p = 6）",
        f"計算 {band_period}",
    )

    values = read_border_table()
    needed_pairs = [
        (n, m)
        for n in range(LARGE_N_FROM, LARGE_N_TO + 1)
        for m in range(n, LARGE_N_TO + 1)
    ]
    missing = [pair for pair in needed_pairs if pair not in values]
    report.check(
        not missing,
        f"大きい行数: {LARGE_N_FROM} ≤ n ≤ m ≤ {LARGE_N_TO} の {len(needed_pairs)} 個の値がそろっている（論文は 496 個）",
        f"ない (n, m): {missing[:5]}",
    )
    report.check(len(needed_pairs) == 496, "大きい行数: 確認する組の数が論文と同じ 496 個")
    mismatches = [
        (pair, values[pair], lower_bound_formula(*pair))
        for pair in needed_pairs
        if pair in values and values[pair] != lower_bound_formula(*pair)
    ]
    report.check(
        not mismatches,
        "大きい行数: 縁の下界が、どの組でも F(n, m) に等しい",
        f"((n, m), 下界, F): {mismatches[:5]}",
    )


def main() -> int:
    report = Report()
    verify_small_heights(report)
    verify_large_heights(report)
    print()
    print("すべて論文と一致" if not report.has_failure else "論文と一致しないものがある")
    return 1 if report.has_failure else 0


if __name__ == "__main__":
    sys.exit(main())
