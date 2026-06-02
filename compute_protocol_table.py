# compute_protocol_table.py
"""Walk three protocol folders, read scaleExperiments_<mode>.csv from each,
average the 3 timing values in each CSV, and print a 3x3 (protocol x mode)
matrix as both a console table and a LaTeX table.

Lives in the Rust project root. Run with: python compute_protocol_table.py
"""
import csv
from pathlib import Path
from statistics import mean


# Project root = where this script lives
PROJECT_ROOT = Path(__file__).resolve().parent

# Protocol label -> folder name on disk
PROTOCOLS = {
    "DH": "basic-diffie-hellman",
    "ORE": "first-interval-intersection",
    "FHE": "fully_HE",
}

MODES = ["natural", "sorted", "shuffled"]


def load_csv_mean(csv_path: Path) -> float:
    """Read all numeric values from a CSV (no header) and return their mean.

    The CSV is expected to be 3 numbers, one per row. Comma-separated values
    on a row are also handled (all numbers across all cells are averaged).
    """
    values = []
    with open(csv_path) as f:
        reader = csv.reader(f)
        for row in reader:
            for cell in row:
                cell = cell.strip()
                if cell:
                    values.append(float(cell))
    if not values:
        raise ValueError(f"{csv_path} contains no numeric values")
    return mean(values)


def main():
    # Build the 3x3 matrix of means (or None where data is missing)
    matrix: dict[str, dict[str, float | None]] = {}

    for protocol_label, folder_name in PROTOCOLS.items():
        matrix[protocol_label] = {}
        folder = PROJECT_ROOT / folder_name

        if not folder.is_dir():
            print(f"WARNING: folder not found: {folder}")
            for mode in MODES:
                matrix[protocol_label][mode] = None
            continue

        for mode in MODES:
            csv_path = folder / f"scaleExperiments_{mode}.csv"
            if not csv_path.exists():
                print(f"WARNING: missing {csv_path.relative_to(PROJECT_ROOT)}")
                matrix[protocol_label][mode] = None
                continue

            try:
                avg = load_csv_mean(csv_path)
                matrix[protocol_label][mode] = avg
            except Exception as e:
                print(f"ERROR reading {csv_path}: {e}")
                matrix[protocol_label][mode] = None

    # --- Console table ---
    print("\n--- Protocol Runtime (ms, mean of 3 runs) ---\n")

    header = ["Protocol"] + [m.capitalize() for m in MODES]
    rows = [header]
    for protocol in PROTOCOLS:
        row = [protocol]
        for mode in MODES:
            val = matrix[protocol][mode]
            row.append(f"{val:.2f}" if val is not None else "—")
        rows.append(row)

    col_widths = [max(len(r[i]) for r in rows) for i in range(len(header))]
    sep = "  ".join("-" * w for w in col_widths)
    for i, row in enumerate(rows):
        line = "  ".join(cell.ljust(col_widths[j]) for j, cell in enumerate(row))
        print(line)
        if i == 0:
            print(sep)

    # --- LaTeX table ---
    print("\n--- LaTeX (booktabs style) ---\n")

    latex_lines = [
        r"\begin{table}[h]",
        r"\centering",
        r"\caption{Protocol runtime across sort modes (ms, mean of 3 runs)}",
        r"\label{tab:protocol-modes}",
        r"\begin{tabular}{lrrr}",
        r"\toprule",
        r"Protocol & Natural & Sorted & Shuffled \\",
        r"\midrule",
    ]
    for protocol in PROTOCOLS:
        cells = [protocol]
        for mode in MODES:
            val = matrix[protocol][mode]
            cells.append(f"{val:.2f}" if val is not None else "---")
        latex_lines.append(" & ".join(cells) + r" \\")
    latex_lines.append(r"\bottomrule")
    latex_lines.append(r"\end{tabular}")
    latex_lines.append(r"\end{table}")

    print("\n".join(latex_lines))


if __name__ == "__main__":
    main()