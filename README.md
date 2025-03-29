# Brainduck

A Brainf*ck parser, written in Rust.

## Operations

- `+`: Increment the current memory cell by 1.
- `-`: Decrement the current memory cell by 1.
- `>`: Move the instruction pointer 1 step to the right.
- `<`: Move the instruction pointer 1 step to the left.
- `.`: Output the contents of the current cell.
- `,`: Write a user specified value to the current cell.
- `[`: Loop begin. If the current cell is non-zero, continue loop.
- `]`: Loop end. Move back to the start of the loop (i.e. matching '[').

## Usage

```rust
$ cargo run -- <path-to-brainf*ck-file>
```

## Custom Features

- `#`: Write a single-line comment. Anything after the `#` will be ignored by the parser.
- `~`: Print the current operation, memory pointer, and memory state. This is useful for debugging code.

## License

Check [LICENSE](LICENSE).
