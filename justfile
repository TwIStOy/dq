alias t := test

debug:
  cargo build --all-targets

release:
  cargo build --all-targets --release

test:
  cargo test --all-targets -- --nocapture
