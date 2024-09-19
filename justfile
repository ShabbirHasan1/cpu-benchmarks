r: run
p: plot

run *args='':
    cargo run -r --quiet -- {{args}}
build:
    @cargo build -r --quiet
record *args='': build
    @perf record ./target/release/perf {{args}}
report:
    @perf report
perf: record report
flame: build
    @cargo flamegraph --open

stat *args='': build
    perf stat -d ./target/release/perf {{args}}

open_plots:
    feh -Z --auto-reload plots/*.png &
plot:
    ./plot.py

cpufreq:
    sudo cpupower frequency-set --governor performance -d 2.6GHz -u 2.6GHz

release: cpufreq
    pkill slack || true
    pkill signal || true
    pkill telegram || true
    pkill discord || true
    pkill htop || true
    pkill chromium || true
    cargo run -r -- --release

watch-thp:
    watch -n 0.1 "grep -E 'trans|thp_fault_alloc' /proc/vmstat"

enable-thp:
    echo always | sudo tee /sys/kernel/mm/transparent_hugepage/enabled
disable-thp:
    echo never | sudo tee /sys/kernel/mm/transparent_hugepage/enabled
