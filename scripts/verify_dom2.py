"""2-domination の計算結果を、Rao と Talon（DMTCS 21(1), 2019）の結果と、results/dom2/ の値で照合する

使い方: python scripts/verify_dom2.py
    先に bash scripts/collect_dom2.sh で results/dom2/ を作っておく

確かめること
    1. 小さい行数（2 ≤ n ≤ 12）
       - period_small.csv の k, p が、Rao と Talon の漸化式 γ2(n, m) = γ2(n, m − k) + p と同じ
       - 漸化式が始まる m は、こちらのほうが 1 遅い（こちらは DP のベクトル全体の一致を見ているため）。違いは表示だけして、失敗にはしない
       - board_small.csv の値が、Rao と Talon の定理 1 の式と同じ。ただし n = 4, 7, 8 は、論文に印刷された式が
         彼ら自身のプログラムの出力と合わないので、直した式と比べる（印刷された式との食い違いの数も表示する）
    2. 大きい行数
       - period_band_c6.txt が r0 = 20, k = 3, p = 6（B^23 = B^20 + 6）
       - border_rect_c6.txt の 13 ≤ n ≤ m ≤ 35（Rao と Talon が確認した範囲）の値がそろっていて、
         計算したすべての値で、縁の下界が ⌊(n+2)(m+2)/3⌋ − 6 に等しい
全部通れば終了コード 0、どれかが違えば 1 で終わる
"""

import csv
import re
import sys
from pathlib import Path

RESULTS = Path(__file__).resolve().parent.parent / "results" / "dom2"


def ceil_div(numerator: int, denominator: int) -> int:
    return -(-numerator // denominator)


def large_grid_formula(n: int, m: int) -> int:
    """Rao と Talon の定理 1 の n ≥ 9 の式 ⌊(n+2)(m+2)/3⌋ − 6"""
    return (n + 2) * (m + 2) // 3 - 6


def printed_formula(n: int, m: int) -> int:
    """Rao と Talon の定理 1 に印刷された式（n = 4, 7, 8 は彼らのプログラムの出力と合わない）"""
    if n == 2:
        return m
    if n == 3:
        return m + ceil_div(m, 3)
    if n == 4:
        return 2 * m - (m // 4) if m % 4 == 3 else 2 * m - (m + 4) // 4
    if n == 5:
        return 2 * m + ceil_div(m, 7) + (1 if m % 7 in (0, 6) else 0)
    if n == 6:
        return 2 * m + (6 * m) // 11 + (1 if m % 11 in (0, 2, 6) else 2)
    if n == 7:
        # 2 つの場合が同じ式で印刷されている
        return 3 * m - m // 18 + 1
    if n == 8:
        return 3 * m + m // 3 + (0 if m % 3 == 1 else 1)
    return large_grid_formula(n, m)


def corrected_formula(n: int, m: int) -> int:
    """印刷された式のうち n = 4, 7, 8 を、彼ら自身の漸化式から導き直したもの（論文の注意に書いた式）"""
    if n == 4:
        return (7 * m + 6) // 4
    if n == 7:
        return (53 * m + 27) // 18 - (1 if m % 18 == 7 else 0) - (1 if m in (8, 9) else 0)
    if n == 8:
        return (10 * m + 4) // 3
    return printed_formula(n, m)


# Rao と Talon の漸化式: n -> (始まる m, k, p)。その m 以上で γ2(n, m) = γ2(n, m − k) + p
PAPER_SMALL_RECURRENCES = {
    2: (3, 1, 1),
    3: (5, 3, 4),
    4: (8, 4, 7),
    5: (14, 7, 15),
    6: (20, 11, 28),
    7: (31, 18, 53),
    8: (16, 3, 10),
    9: (17, 3, 11),
    10: (14, 1, 4),
    11: (16, 3, 13),
    12: (17, 3, 14),
}

# 幅 6 の帯の行列: B^(r0 + k) = B^r0 + p
EXPECTED_BAND_PERIOD = (20, 3, 6)

# Rao と Talon が縁の下界を確認した範囲
LARGE_N_FROM = 13
LARGE_N_TO = 35


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

    def note(self, description: str) -> None:
        """成否に関係しない情報"""
        print(f"[--] {description}")


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
    text = (RESULTS / "period_band_c6.txt").read_text(encoding="utf-8")
    found = re.search(r"^r0 = (\d+), k = (\d+), p = (-?\d+)$", text, re.MULTILINE)
    if found is None:
        return None
    return int(found.group(1)), int(found.group(2)), int(found.group(3))


def read_border_table() -> dict[tuple[int, int], int]:
    """border_sweep.sh の表（1 行目が m、各行の先頭が n、計算しないところは「-」）"""
    lines = (RESULTS / "border_rect_c6.txt").read_text(encoding="utf-8").splitlines()
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
    for n, (paper_start_m, paper_k, paper_p) in PAPER_SMALL_RECURRENCES.items():
        computed = computed_periods.get(n)
        if computed is None:
            report.check(False, f"小さい行数 n = {n}: 周期が計算されている")
            continue
        m0, k, p = computed
        report.check(
            (k, p) == (paper_k, paper_p),
            f"小さい行数 n = {n}: 漸化式の k, p が論文と同じ（k = {k}, p = {p}）",
            f"計算 (k, p) = {(k, p)}、論文 {(paper_k, paper_p)}",
        )
        # こちらの漸化式 γ2(n, m) = γ2(n, m − k) + p は m − k ≥ m0、つまり m ≥ m0 + k から
        report.note(
            f"小さい行数 n = {n}: 漸化式が始まる m は、こちら {m0 + k}、論文 {paper_start_m}"
        )

    values = read_small_values()
    for n in PAPER_SMALL_RECURRENCES:
        ms = [m for (value_n, m) in values if value_n == n]
        mismatches = [
            (m, values[(n, m)], corrected_formula(n, m))
            for m in sorted(ms)
            if values[(n, m)] != corrected_formula(n, m)
        ]
        label = "直した式" if n in (4, 7, 8) else "論文の式"
        report.check(
            not mismatches,
            f"小さい行数 n = {n}: 計算したすべての値（m ≤ {max(ms)}）が{label}と同じ",
            f"(m, 計算, 式): {mismatches[:5]}",
        )
        if n in (4, 7, 8):
            printed_mismatch_count = sum(
                1 for m in ms if values[(n, m)] != printed_formula(n, m)
            )
            report.note(
                f"小さい行数 n = {n}: 論文に印刷された式とは {printed_mismatch_count} 個で食い違う"
            )


def verify_large_heights(report: Report) -> None:
    band_period = read_band_period()
    report.check(
        band_period == EXPECTED_BAND_PERIOD,
        "大きい行数: 幅 6 の帯の行列が B^23 = B^20 + 6（r0 = 20, k = 3, p = 6）",
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
        f"大きい行数: 論文の確認範囲 {LARGE_N_FROM} ≤ n ≤ m ≤ {LARGE_N_TO} の {len(needed_pairs)} 個の値がそろっている",
        f"ない (n, m): {missing[:5]}",
    )
    mismatches = [
        (pair, lower_bound, large_grid_formula(*pair))
        for pair, lower_bound in sorted(values.items())
        if lower_bound != large_grid_formula(*pair)
    ]
    largest = max(max(pair) for pair in values)
    report.check(
        not mismatches,
        f"大きい行数: 計算したすべての組（{len(values)} 個、n, m ≤ {largest}）で、縁の下界が ⌊(n+2)(m+2)/3⌋ − 6 に等しい",
        f"((n, m), 下界, 式): {mismatches[:5]}",
    )


def main() -> int:
    report = Report()
    verify_small_heights(report)
    verify_large_heights(report)
    print()
    print("すべて Rao と Talon の結果と一致" if not report.has_failure else "一致しないものがある")
    return 1 if report.has_failure else 0


if __name__ == "__main__":
    sys.exit(main())
