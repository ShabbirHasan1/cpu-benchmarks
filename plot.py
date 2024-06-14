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

# Create a figure
fig, ax = plt.subplots()
ax.set_title("Pointer Chasing")
ax.set_xlabel("Array Size (bytes)")
ax.set_ylabel("Latency (ns)")
# Plot a single line for each experiment.
for name in data.name.unique():
    ax.plot("sz", "latency", data=data[data.name == name], label=name)
ax.set_xscale("log", base=2)
ax.set_ylim(0, None)
ax.yaxis.grid(True)
ax.legend()

# Add vertical lines for cache sizes
ax.axvline(x=l1_size, color="red", linestyle="--")
ax.axvline(x=l2_size, color="red", linestyle="--")
if l3_size <= data.sz.max():
    ax.axvline(x=l3_size, color="red", linestyle="--")

# Save
fig.savefig("plots/pointer-chasing.png")
fig.savefig("plots/pointer-chasing.svg")
