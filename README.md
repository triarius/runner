Runner
===

A process runner that can log the `stdin`, `stdout`, and `stderr` streams to files.

# Usage
```
A command runner that optionally logs the I/O streams to files

Usage: runner [OPTIONS] -- <EXEC>...

Arguments:
  <EXEC>...  The command to run and its arguments. A command must be specified, arguments are space delimited

Options:
  -i, --in-file <IN_FILE>    The file to log stdin to [env: IN_FILE=]
  -o, --out-file <OUT_FILE>  The file to log stdout to [env: OUT_FILE=]
  -e, --err-file <ERR_FILE>  The file to log stderr to [env: ERR_FILE=]
      --no-header            Whether to write a header to the log file(s) [env: NO_HEADER=]
  -h, --help                 Print help
  -V, --version              Print version
```
