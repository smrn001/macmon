# macmon

Ultra-lightweight, keyboard-driven system monitor for macOS, built in Rust.

```bash
macmon
```

macmon focuses on the information developers and power users need most — CPU,
memory, processes, network and disk activity — through a fast terminal UI with
a minimal operational footprint:

> Do less, use less, stay fast.

## Features

- Total and per-core CPU usage
- Memory usage: used, cached, compressed, swap
- Process list with CPU/memory sorting and live search-free navigation
- Per-process details: user, parent PID, threads, state, executable path
- Kill processes with confirmation (SIGTERM)
- Network throughput: download/upload rates and totals
- Disk usage and read/write throughput

## Install

### Homebrew

```bash
brew install macmon
```

### Cargo

```bash
cargo install macmon
```

### GitHub Releases

Prebuilt binaries for Apple Silicon and Intel are available on the
[releases page](https://github.com/smrn001/macmon/releases):

```bash
curl -LO https://github.com/smrn001/macmon/releases/latest/download/macmon-aarch64-apple-darwin.tar.gz
tar xzf macmon-aarch64-apple-darwin.tar.gz
sudo mv macmon /usr/local/bin/
```

## Usage

```bash
macmon
```

### Keybindings

| Key | Action |
|-----|-------------------------|
| `↑` / `k` | Select previous process |
| `↓` / `j` | Select next process |
| `c` | Sort by CPU |
| `m` | Sort by memory |
| `p` | Sort by PID |
| `a` | Sort by name |
| `Enter` | Open process details |
| `k` | Kill selected process (in details view) |
| `y` / `n` | Confirm / cancel kill |
| `b` / `Esc` | Back |
| `q` | Quit |

## Performance

Performance is a first-class requirement. Targets:

| Metric | Target |
|--------------------|-----------|
| Idle CPU | < 0.5% |
| Memory (RSS) | < 15 MB |
| Startup | < 50 ms |

Benchmark results against `top` and Activity Monitor are tracked in the
repository documentation.

## Building from source

Requires a Rust toolchain for `aarch64-apple-darwin` or
`x86_64-apple-darwin`:

```bash
cargo build --release
```

macOS only. No background daemon, no telemetry, no network access, no cloud.

## License

MIT — see [LICENSE](LICENSE).
