use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let config = match parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("logsquash: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let reader = match build_reader(&config.paths) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("logsquash: {}", e);
            return ExitCode::FAILURE;
        }
    };

    let stdout = io::stdout();
    let mut out = stdout.lock();

    if let Err(e) = squash(reader, config.skip, config.count_only, &mut out) {
        eprintln!("logsquash: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

struct Config {
    skip: usize,
    count_only: bool,
    paths: Vec<String>,
}

// --skip/-s takes the number of leading characters to ignore when comparing
// lines - enough to cover a fixed-width timestamp prefix - without touching
// what actually gets printed. --count-only/-c drops the line text from the
// output entirely, leaving just the repeat count per group. Everything else
// is treated as a file path.
fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut skip: usize = 0;
    let mut count_only = false;
    let mut paths = Vec::new();
    let mut iter = args.iter();

    while let Some(arg) = iter.next() {
        if arg == "-" {
            paths.push(arg.clone());
            continue;
        }

        let (flag, inline_value) = match arg.split_once('=') {
            Some((f, v)) => (f, Some(v.to_string())),
            None => (arg.as_str(), None),
        };

        match flag {
            "--skip" | "-s" => {
                let value = match inline_value {
                    Some(v) => v,
                    None => iter
                        .next()
                        .ok_or_else(|| format!("{} requires a value", flag))?
                        .clone(),
                };
                skip = value
                    .parse()
                    .map_err(|_| format!("invalid --skip value '{}'", value))?;
            }
            "--count-only" | "-c" => {
                if inline_value.is_some() {
                    return Err(format!("{} takes no value", flag));
                }
                count_only = true;
            }
            _ if arg.starts_with('-') => {
                return Err(format!("unknown option '{}'", arg));
            }
            _ => paths.push(arg.clone()),
        }
    }

    Ok(Config {
        skip,
        count_only,
        paths,
    })
}

// With no args (or a lone "-") we read stdin so this works in a pipeline.
// Otherwise every argument is treated as a file path and read in order,
// as if they'd been concatenated first - that keeps squashing correct
// across a run that spans a file boundary.
fn build_reader(args: &[String]) -> io::Result<Box<dyn BufRead>> {
    if args.is_empty() || (args.len() == 1 && args[0] == "-") {
        return Ok(Box::new(BufReader::new(io::stdin())));
    }

    let mut combined: Box<dyn Read> = Box::new(io::empty());
    for path in args {
        let next: Box<dyn Read> = if path == "-" {
            Box::new(io::stdin())
        } else {
            let file = File::open(path)
                .map_err(|e| io::Error::new(e.kind(), format!("cannot open '{}': {}", path, e)))?;
            Box::new(file)
        };
        combined = Box::new(combined.chain(next));
    }

    Ok(Box::new(BufReader::new(combined)))
}

fn squash(reader: impl BufRead, skip: usize, count_only: bool, out: &mut impl Write) -> io::Result<()> {
    let mut prev: Option<String> = None;
    let mut count: usize = 0;

    for line in reader.lines() {
        let line = line?;
        match &prev {
            Some(p) if compare_key(p, skip) == compare_key(&line, skip) => count += 1,
            Some(p) => {
                flush(out, p, count, count_only)?;
                prev = Some(line);
                count = 1;
            }
            None => {
                prev = Some(line);
                count = 1;
            }
        }
    }

    if let Some(p) = prev {
        flush(out, &p, count, count_only)?;
    }

    Ok(())
}

// Skips by character, not byte, so multi-byte UTF-8 in the timestamp itself
// doesn't land us mid-character. A line shorter than `skip` compares as
// empty rather than panicking or falling back to the full line.
fn compare_key(line: &str, skip: usize) -> &str {
    match line.char_indices().nth(skip) {
        Some((idx, _)) => &line[idx..],
        None => "",
    }
}

fn flush(out: &mut impl Write, line: &str, count: usize, count_only: bool) -> io::Result<()> {
    if count_only {
        return writeln!(out, "{}", count);
    }
    if count > 1 {
        writeln!(out, "{}  (x{})", line, count)
    } else {
        writeln!(out, "{}", line)
    }
}
