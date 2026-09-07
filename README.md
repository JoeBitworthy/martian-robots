# Martian Robots

Robots move around a bounded rectangular grid on Mars following `L`, `R` and
`F` instructions. A robot that moves off an edge is lost, but leaves a scent on
the cell it fell from, and later robots ignore any instruction that would take
them off the grid from a scented cell. The sample from the brief is in
`testdata/`.

## Run

    cargo run < testdata/sample_input.txt

Needs a Rust toolchain (1.85 or newer) and nothing else; https://rustup.rs
installs one on macOS, Linux or Windows. With Docker and no Rust at all:

    docker run --rm -i -v "$PWD":/app -w /app rust:1-slim cargo run -q < testdata/sample_input.txt

Invalid input is reported with its line number on stderr and exit code 1. CI
runs the tests and the sample on all three operating systems.

## Test

    cargo test

## How it is put together

- **Pure core, thin shell.** Everything in `src/` except `main.rs` is plain
  values and functions with no I/O, so every rule has a unit test next to it
  and `run()` in `lib.rs` is the whole program as a function from text to
  text.
- **Commands are a closed enum.** Adding an instruction is a new variant in
  `command.rs`, a parser arm and an arm in `Robot::apply`; the compiler
  refuses to build until every `match` handles it. `DESIGN.md` explains why
  an enum rather than a trait.
- **The world owns the scents.** `World::execute` is the only code that knows
  about edges and scents, so the lost-robot rule lives in exactly one place.
- **Input and output stream.** Robots are read, run and written one at a
  time, so memory does not depend on the size of the input.
- **No dependencies, no persistence, no UI.** Nothing outlives one run.

## A note on the sample data

The third robot only survives because the second was lost from (3, 3): its
`F` towards the north edge is ignored and it ends at `2 3 S`. Without scents
it would read `3 3 N LOST`, so the sample doubles as a regression test for the
hardest rule in the brief. `tests/sample.rs` checks the full output and that
the rescue really is the scent's doing.

## Next steps

- Report an error per robot rather than stopping at the first bad line.
- A `--json` output flag for machine consumers.
- Fuzz the parser.
