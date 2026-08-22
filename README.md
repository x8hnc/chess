
# Chess Engine

A chess engine with both a web-based UI and a terminal TUI. The engine supports configurable search depth and thread count.

## Usage

Run the engine with:

```bash
cargo run -- [options]
```

By default, the engine starts the **Web UI** on:

```text
http://127.0.0.1:8585
```

### Options

| Flag | Argument    | Description                                         | Default |
| ---- | ----------- | --------------------------------------------------- | ------- |
| `-b` | —           | Play as Black                                       | White   |
| `-t` | `<threads>` | Set the number of search threads                    | `20`    |
| `-d` | `<depth>`   | Set the search depth                                | `6`     |
| `-f` | —           | Force the board to display with White at the bottom | Off     |
| `-h` | —           | Start the terminal UI instead of the Web UI         | Web UI  |

Arguments cannot be specified more than once.

### Examples

Start the default Web UI:

```bash
cargo run
```

Play as Black:

```bash
cargo run -- -b
```

Use 8 threads and a search depth of 10:

```bash
cargo run -- -t 8 -d 10
```

Play as Black with White forced to the bottom of the board:

```bash
cargo run -- -b -f
```

Start the TUI with 12 threads and depth 8:

```bash
cargo run -- -h -t 12 -d 8
```

## Web UI

In Web UI mode, make a move by:

1. Clicking the piece you want to move.
2. Clicking the square you want to move it to.

Press **`R`** to reset the game.

## TUI

In TUI mode, moves are entered using **UCI notation**.

For example:

```text
e2e4
g8f6
```

You can also enter:

```text
reset
```
to reset the game.

## Defaults

* **Threads:** 20
* **Search depth:** 6
* **Side:** White
* **Board orientation:** White at the bottom
* **Interface:** Web UI
* **Web address:** `127.0.0.1:8585`
