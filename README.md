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

Output:

```
2026-09-19T10:00:01Z WARN retry queue full, dropping oldest entry
2026-09-19T10:00:02Z WARN retry queue full, dropping oldest entry
2026-09-19T10:00:03Z WARN retry queue full, dropping oldest entry
2026-09-19T10:00:04Z ERROR connection reset by peer
```

Note that these four lines each differ by their timestamp, so none of them
collapse yet - that's the main limitation right now, see below.

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

## Current limitation

Lines are compared for exact equality, so a timestamp prefix that changes
every line defeats the squash. A log line with no timestamp, or one where you
strip the timestamp before piping it in, squashes as expected:

```
cut -d' ' -f2- app.log | logsquash
```

Stripping a configurable timestamp prefix automatically is the next piece of
work - see the roadmap in the repo.

## Build

Standard library only, nothing to fetch:

```
cargo build --release
```
