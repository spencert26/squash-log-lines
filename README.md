# logsquash

A service that logs the same warning every second buries the one line you
actually need under a thousand copies of it. `logsquash` collapses runs of
consecutive, identical log lines into a single line with a repeat count, the
way `uniq -c` does, but formatted for reading log output rather than piping
into further scripts.

Input line:

```
2026-09-19T10:00:01Z WARN retry queue full, dropping oldest entry
2026-09-19T10:00:02Z WARN retry queue full, dropping oldest entry
2026-09-19T10:00:03Z WARN retry queue full, dropping oldest entry
2026-09-19T10:00:04Z ERROR connection reset by peer
```

Those first three lines differ only by timestamp, so a plain equality check
never collapses them. `--skip` tells logsquash how many leading characters to
ignore when comparing lines, without dropping them from the output:

```
logsquash --skip 21 app.log
```

Output:

```
2026-09-19T10:00:01Z WARN retry queue full, dropping oldest entry  (x3)
2026-09-19T10:00:04Z ERROR connection reset by peer
```

The printed line is always the first one seen in the run, timestamp and all -
only the comparison ignores the prefix. 21 is the width of
`2026-09-19T10:00:01Z ` (20-character ISO 8601 timestamp plus the trailing
space); adjust it to match your own log format.

## Usage

Read a file:

```
logsquash app.log
```

Read from stdin, so it drops into a pipeline:

```
tail -f app.log | logsquash
journalctl -u myservice | logsquash -
```

Read several files as one stream, in order:

```
logsquash app.log.1 app.log
```

Ignore a leading timestamp (or any other fixed-width prefix) when deciding
whether two lines match:

```
logsquash --skip 21 app.log
tail -f app.log | logsquash -s 21
```

`--skip` counts characters, not bytes, and `-s` is a shorthand for the same
flag. `--skip=21` works too.

## Build

Standard library only, nothing to fetch:

```
cargo build --release
```
