set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

export CARGO_INCREMENTAL := "0"
export RUSTFLAGS := "-Cinstrument-coverage"
export LLVM_PROFILE_FILE := "cargo-test-%p-%m.profraw"

cov:
    cargo llvm-cov nextest --lcov --output-path=lcov.info

cov-ci:    
    cargo llvm-cov nextest --lcov --output-path=lcov.info

watch:
    cargo watch -x "llvm-cov nextest --lcov --output-path=lcov.info"