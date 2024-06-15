#!/usr/bin/env python3
import pandas
import matplotlib.pyplot as plt

l1_size = 192 * 1024 / 6
l2_size = 1.5 * 1024 * 1024 / 6
l3_size = 12 * 1024 * 1024

filename = "results/pointer-chasing.json"
data = pandas.read_json(filename)
data["sz"] = data["size"]

data = data[data.sz >= l1_size // 4]


def plot(data, fig_name):
    # Create a figure
    fig, ax = plt.subplots()
    ax.set_title("Pointer Chasing")
    ax.set_xlabel("Array Size (bytes)")
    ax.set_ylabel("Latency (NS)")
    # Plot a single line for each experiment.
    for name in data.name.unique():
        ax.plot(
            "sz",
            "latency",
            data=data[data.name == name],
            label=name,
            linestyle="--",
            linewidth=1,
        )
    ax.set_xscale("log", base=2)
    ax.set_ylim(0, None)
    ax.yaxis.grid(True)
    # legend on the right
    ax.legend(loc="upper right", bbox_to_anchor=(1, 1))

    # Add vertical lines for cache sizes
    ax.axvline(x=l1_size, color="red", linestyle="--")
    ax.axvline(x=l2_size, color="red", linestyle="--")
    if l3_size <= 2 * data.sz.max():
        ax.axvline(x=l3_size, color="red", linestyle="--")

    # Save
    fig.savefig(f"plots/{fig_name}.png", bbox_inches="tight", dpi=600)
    fig.savefig(f"plots/{fig_name}.svg", bbox_inches="tight")


plot(data[~data.name.str.contains("work")], "pointer-chasing")
plot(data[data.name.str.contains("work")], "pointer-chasing-work")
