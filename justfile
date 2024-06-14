p: plot

run *args='':
    @cargo run -r --quiet -- {{args}}
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
    feh --auto-reload plots/* &
plot:
    ./plot.py
