# termlog

A tiny tool to log inputs and outputs of terminal stripping control sequenses in combination with tmux

## Build

```
git clone https://github.com/bashi8128/termlog
cd termlog
cargo build --release 
mkdir -p ~/.local/bin/ && mv ./target/release/termlog ~/.local/bin/
```

## Usage

With the following command in a tmux session, write the terminal’s input and output to a file under ~/log/YYYY/MM/DD.

```
tmux pipe-pane -o "${HOME}/.local/bin/termlog"
```
