set shell := ["powershell.exe", "-c"]

export CARGO_INCREMENTAL := "0"
export RUSTFLAGS := "-Cinstrument-coverage"
export LLVM_PROFILE_FILE := "cargo-test-%p-%m.profraw"

cov:
    cargo llvm-cov nextest --lcov --output-path=lcov.info
