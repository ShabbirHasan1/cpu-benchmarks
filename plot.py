#!/usr/bin/env python3
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
from pathlib import Path
import tabulate

l1_size = 192 * 1024 / 6
l2_size = 1.5 * 1024 * 1024 / 6
l3_size = 12 * 1024 * 1024

palette = None
dashes = {"": "", "Latency": (1, 1), "Prefetch": (2, 1)}


def plot(experiment_name, title, data, names, skip=0, ymax=85, latency=False):
    # Create a figure
    fig, ax = plt.subplots(figsize=(8, 6))
    ax.set_title(title)
    ax.set_xlabel("Array Size (bytes)")
    if latency:
        ax.set_ylabel("Latency (ns)")
    else:
        ax.set_ylabel("Inverse throughput (ns)")

    global dashes
    for s in data.Style.unique():
        assert s in dashes

    data = data[data["name"].isin(names)]
    sns.lineplot(
        x="sz",
        y="latency",
        hue=data[
            "Color" if len(data.batchsize.unique()) == 1 else "display_name"
        ].tolist(),
        style="Style",  # if data.Style.unique().tolist() != [""] else None,
        dashes=dashes,
        data=data,
        legend="auto",
        sizes=[2, 3, 4, 5, 6, 7, 8, 9],
        palette=palette,
        errorbar=("pi", 100),
        estimator="median",
    )

    ax.set_xscale("log", base=2)
    ax.set_ylim(0, ymax)
    ax.grid(True, alpha=0.4)
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
    return fig


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

    best_latency = data[data.name == "raw_pointer_chasing_padded_aligned"].copy()
    best_latency["name"] = "latency"
    data = pd.concat([best_latency, data])

    data["sz"] = data["size"]
    data["display_name"] = (
        data["name"].str.replace("_", " ").str.replace(" prefetch", "").str.title()
    )
    data["display_size"] = data["sz"].apply(display_size)

    def style(name):
        if "prefetch" in name:
            return "Prefetch"
        if "latency" in name:
            return "Latency"
        else:
            return ""

    def batchsize(name):
        if "batch" in name:
            return int(name.split("<")[1].split(">")[0])
        return 0

    def color(name):
        name = name.split("<")[0]
        return name

    data["Style"] = [style(name) for name in data.name]
    data["batchsize"] = [batchsize(name) for name in data.name]
    data["Color"] = [color(name) for name in data.display_name]

    global palette

    names = sorted(data.display_name.unique())
    colors = sns.color_palette(n_colors=10)
    colors = colors + colors + colors + colors
    palette = dict(zip(names, colors))
    palette["Latency"] = "black"

    return data


# Read all files in the 'results' directory and iterate over them.
def plot_latency():
    data = read_file(f"results/latency-release.json")
    names = ["pointer_chasing_checked"]
    plot("latency-1", "Pointer chasing latency", data, names, latency=True)
    names.append("pointer_chasing")
    plot("latency-2", "Unchecked indexing", data, names, latency=True)
    # Drop checked indexing.
    names.remove("pointer_chasing_checked")

    names.append("pointer_chasing_padded")
    plot(
        "latency-3",
        "Padding to 64 bytes cachelines",
        data,
        names,
        skip=1,
        latency=True,
    )
    # Drop non-packed
    names.remove("pointer_chasing")
    names.append("raw_pointer_chasing_padded")
    plot("latency-4", "Using raw pointers", data, names, skip=2, latency=True)

    # Add aligned
    names.append("pointer_chasing_padded_aligned")
    names.append("raw_pointer_chasing_padded_aligned")

    plot("latency-5", "2MB aligned hugepages", data, names, skip=2, latency=True)

    summary_table(data)


def plot_batch():
    latency_data = read_file(f"results/latency-release.json")
    batch_data = read_file(f"results/batch-release.json")
    data = pd.concat([latency_data, batch_data])

    all_names = data.name.unique()
    batch_names = [
        "latency",
        "batch<1>",
        "batch<2>",
        "batch<4>",
        "batch<4>",
        "batch<8>",
        "batch<16>",
        "batch<32>",
    ]
    plot("batch-1", "Batch", data, batch_names, skip=5)
    names = [
        "latency",
        "batch<10>",
        "batch<11>",
        "batch<12>",
        "batch<13>",
        "batch<16>",
    ]
    plot("batch-2", "Saturating batch size", data, names, skip=-4, ymax=9)

    names = ["latency", "batch<16>"] + [
        name for name in all_names if "batch_work" in name
    ]
    plot(
        "batch-3",
        "Batching with work",
        data,
        names,
        skip=0,
    )

    names = ["latency", "batch<16>", "batch_prefetch<16>"] + [
        name for name in all_names if "work" in name
    ]
    plot(
        "batch-4",
        "Prefetching",
        data,
        names,
        skip=0,
    )


plt.close("all")
plot_latency()
plot_batch()
