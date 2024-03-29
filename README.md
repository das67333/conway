# Conway's Game of Life

The egui (immediate-mode GUI Rust framework) is used to draw the graphics. GUI elements are going to be added after tuning memory consumpsion.

## Running

```bash
cargo run --release
```

## Controls

- <kbd>P</kbd>: Toggle pause.
- <kbd>space</kbd>: Frame step (enables pause if not already paused)
- <kbd>R</kbd>: Randomize
- <kbd>F</kbd>: Do 100 updates per frame
- <kbd>escape</kbd>: Quit

## Benchmarks

```bash
cargo bench
```

Reports are in `target/criterion/`


## Tests

```bash
cargo test
```
