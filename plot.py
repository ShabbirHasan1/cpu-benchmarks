#!/usr/bin/env python3
import pandas
import matplotlib.pyplot as plt

l1_size = 192 * 1024 / 6
l2_size = 1.5 * 1024 * 1024 / 6
l3_size = 12 * 1024 * 1024

filename = "results/pointer-chasing.json"
data = pandas.read_json(filename)

# Create a figure
fig, ax = plt.subplots()
ax.set_title("Pointer Chasing")
ax.set_xlabel("Array Size (bytes)")
ax.set_ylabel("Latency (ns)")
ax.plot("n", "latency", data=data)
ax.set_xscale("log", base=2)
ax.set_ylim(0, None)
# Add horizontal grid lines
ax.yaxis.grid(True)

# Add vertical lines for cache sizes
ax.axvline(x=l1_size, color="red", linestyle="--")
ax.axvline(x=l2_size, color="red", linestyle="--")
ax.axvline(x=l3_size, color="red", linestyle="--")

# Save
fig.savefig("plots/pointer-chasing.png")

# Zoom in
xmax = 2 * l3_size
ax.set_xlim(None, xmax)

# Find max y of x<=xmax
ymax = data[data.n <= xmax].latency.max() * 1.1
ax.set_ylim(ymin=0, ymax=ymax)
fig.savefig("plots/pointer-chasing-zoom.png")
