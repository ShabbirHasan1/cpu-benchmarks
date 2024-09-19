#!/usr/bin/env python3
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
from pathlib import Path
import tabulate

l1_size = 192 * 1024 / 6
l2_size = 1.5 * 1024 * 1024 / 6
l3_size = 12 * 1024 * 1024


def plot(experiment_name, title, data, skip=0):
    # Create a figure
    fig, ax = plt.subplots(figsize=(8, 6))
    ax.set_title(title)
    ax.set_xlabel("Array Size (bytes)")
    ax.set_ylabel("Latency (ns)")
    # for _ in range(skip):
    #     ax.plot([], [])

    colors = sns.color_palette(n_colors=6)[skip::] + sns.color_palette()

    sns.lineplot(
        x="sz",
        y="latency",
        hue="display_name",
        data=data,
        legend="full",
        marker=None,
        dashes=False,
        # label=name.replace("_", " ").title(),
        # linestyle="--" if "prefetch" in name else "-",
        linewidth=1,
        palette=colors,
        errorbar=("pi", 100),
        estimator="median",
    )

    # Plot a single line for each experiment.
    # for name in data.name.unique():
    #     ax.plot(
    #         "sz",
    #         "latency",
    #         data=data[data.name == name],
    #         label=name.replace("_", " ").title(),
    #         linestyle="--" if "prefetch" in name else "-",
    #         linewidth=1,
    #     )
    ax.set_xscale("log", base=2)
    ax.set_ylim(0, 85)
    ax.yaxis.grid(True)
    # legend on the right
    ax.legend(loc="upper left")

    # Add vertical lines for cache sizes
    for size, name in [(l1_size, "L1"), (l2_size, "L2"), (l3_size, "L3")]:
        ax.axvline(x=size, color="red", linestyle="--", zorder=0)
        ax.text(size, 0, f"{name} ", color="red", va="bottom", ha="right")

    #
    if l3_size < data.sz.max():
        ax.text(data.sz.max(), 0, "RAM", color="red", va="bottom", ha="right")

    # Save
    fig.savefig(f"plots/{experiment_name}.png", bbox_inches="tight", dpi=600)
    print(f"Saved {experiment_name}.png")
    fig.savefig(f"plots/{experiment_name}.svg", bbox_inches="tight")
    print(f"Saved {experiment_name}.svg")


def summary_table(data):
    # Table of statistics at a few key sizes.
    szs = [l1_size / 2, l2_size / 2, l3_size / 3, data.sz.max()]
    names = ["L1", "L2", "L3", "RAM"]
    data = data[data.sz.isin(szs)]
    table = pd.pivot_table(
        data,
        index="display_name",
        columns="display_size",
        values=["latency", "cycles"],
        sort=False,
    )
    print(
        tabulate.tabulate(
            table, headers=table.columns, tablefmt="orgtbl", floatfmt=".1f"
        )
    )


def display_size(size):
    if size <= l1_size:
        return "L1"
    if size <= l2_size:
        return "L2"
    if size <= l3_size:
        return "L3"
    return "RAM"


def read_file(filename):
    data = pd.read_json(filename)
    data["sz"] = data["size"]
    data["display_name"] = data["name"].str.replace("_", " ").str.title()
    data["display_size"] = data["sz"].apply(display_size)
    return data


# Read all files in the 'results' directory and iterate over them.
def plot_latency():
    data = read_file(f"results/latency-big.json")
    names = ["pointer_chasing_checked"]
    plot("latency-1", "Pointer chasing latency", data[data["name"].isin(names)])
    names.append("pointer_chasing")
    plot("latency-2", "Unchecked indexing", data[data["name"].isin(names)])
    # Drop checked indexing.
    names.remove("pointer_chasing_checked")

    names.append("pointer_chasing_padded")
    plot(
        "latency-3",
        "Padding to 64 bytes cachelines",
        data[data["name"].isin(names)],
        skip=1,
    )
    # Drop non-packed
    names.remove("pointer_chasing")
    names.append("raw_pointer_chasing_padded")
    plot("latency-4", "Using raw pointers", data[data["name"].isin(names)], skip=2)
    names.remove("raw_pointer_chasing_padded")

    # Add aligned
    names.append("pointer_chasing_padded_aligned")

    plot("latency-5", "2MB aligned hugepages", data[data["name"].isin(names)], skip=3)

    summary_table(data)


plot_latency()
