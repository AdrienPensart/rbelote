Belote

Crates :
- core : the rules engine, no user interface; `bot` holds the rules of thumb
  bots play by, and a random player for tests
- cli : terminal front end, `cargo run -p rbelote-cli`
- gui : desktop front end drawn with egui (immediate mode, wgpu/Metal on macOS),
  `cargo run -p rbelote-gui --release`

Usage :
- test :
  - cargo test --workspace
- linting :
  - clippy : cargo clippy --all-targets --all-features -- -D warnings
