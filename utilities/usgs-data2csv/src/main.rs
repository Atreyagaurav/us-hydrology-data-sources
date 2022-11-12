use clap::Parser;
use flate2::read::GzDecoder as GzDecoderRead;
use flate2::write::GzDecoder as GzDecoderWrite;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Lines, Write};
use std::path::PathBuf;
use string_join::Join;

#[derive(Parser)]
#[clap(about = "Converts the tab sep data from usgs website to csv format.")]
struct Cli {
    /// Comment Character
    #[clap(short, long, default_value = "#")]
    comment_chars: char,
    /// Delineate Character In input file
    #[clap(short = 'D', long, default_value = "\t")]
    input_deliminator: char,
    /// Delineate Character in output file
    #[clap(short, long, default_value = ",")]
    delineate: char,
    /// Echo Contents (Skips comments)
    #[clap(short, long)]
    echo: bool,
    /// Ignore Errors
    #[clap(short, long)]
    ignore_errors: bool,
    /// Output file to write instead of stdout
    #[clap(short, long)]
    output: Option<PathBuf>,
    /// Number of head lines to print
    #[clap(short, long)]
    head: Option<u64>,
    /// Print the names of the columns with number and exit
    #[clap(short, long)]
    names: bool,
    /// Specific line numbers to skip (Comments not counted)
    #[clap(short, long, value_delimiter = ',', value_name = "N")]
    skip: Vec<u64>,
    /// Only output these column numbers
    #[clap(short = 'C', long, value_delimiter = ',', value_name = "N")]
    columns: Option<Vec<usize>>,
    /// TODO Split by this Column and write into separate files, requires output
    #[clap(short = 'S', long, requires = "output", value_name = "N")]
    split: Option<usize>,
    /// Filter by value in a Column
    #[clap(short = 'f', long, value_delimiter = ',', value_name = "N:VAL")]
    filter: Option<Vec<String>>,
    /// TODO Recursively parse directories, otherwise skip directories
    #[clap(short, long)]
    recursive: bool,
    /// File list to be parsed
    #[clap(required = true)]
    files: Vec<PathBuf>,
}

fn sanitize_cell(raw: &str) -> String {
    let cell = raw.trim();
    if cell.contains(",") {
        return format!("\"{}\"", cell);
    } else {
        return cell.to_string();
    }
}

fn write_output_check_err(out: &mut Option<BufWriter<File>>, line: &str) -> bool {
    match out {
        // couldn't make std::io::stdout and the BufWriter into a
        // single variable. even though both can be used to write into
        // well, had the same problem with input when I wanted to
        // support both normal text file and Gzipped file, So I think
        // I can do it later with same type of enum.
        Some(r) => writeln!(r, "{}", line).is_err(),
        None => writeln!(std::io::stdout(), "{}", line).is_err(),
    }
}

enum ReadFileFormat {
    // Stdout,
    Text(Lines<BufReader<File>>),
    Gzip(Lines<BufReader<GzDecoderRead<File>>>),
}

enum WriteFileFormat {
    // Stdin,
    Text(Lines<BufWriter<File>>),
    Gzip(Lines<BufWriter<GzDecoderWrite<File>>>),
}

impl ReadFileFormat {
    fn text(filename: PathBuf) -> Self {
        let file = File::open(&filename).unwrap();
        ReadFileFormat::Text(BufReader::new(file).lines())
    }
    fn gzip(filename: PathBuf) -> Self {
        let file = File::open(&filename).unwrap();
        ReadFileFormat::Gzip(BufReader::new(GzDecoderRead::new(file)).lines())
    }
}

impl Iterator for ReadFileFormat {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        match self {
            // TODO ReadFileFormat::Stdio => None, // read stdin,
            ReadFileFormat::Text(lines) => lines.next()?.ok(),
            ReadFileFormat::Gzip(lines) => lines.next()?.ok(),
        }
    }
}

fn main() {
    let args: Cli = Cli::parse();
    let mut file: ReadFileFormat;
    let mut count = 0u64;
    let mut skipped = 0u64;
    let mut col_len: Option<usize> = None;
    let mut output = None;

    if args.output.is_some() && args.split.is_none() {
        // output is a file, otherwise it's a directory for split files.
        output = Some(BufWriter::new(
            File::create(args.output.unwrap()).expect("Cannot open output file."),
        ));
    }

    let mut filter_vals = HashMap::<usize, String>::new();
    if let Some(ref filter) = args.filter {
        let mut k: usize;
        for kvf in filter {
            let mut kv = kvf.split(":");
            k = kv
                .next()
                .expect("No Key Column for Filtering")
                .parse()
                .expect("The Key is not a Column number");
            filter_vals.insert(
                k,
                kv.next()
                    .expect("The value to Filter the column is empty")
                    .to_string(),
            );
        }
    }

    let mut output_line;

    for filename in args.files {
        if !filename.is_file() {
            continue;
        }
        if filename.extension().and_then(OsStr::to_str) == Some("gz") {
            file = ReadFileFormat::gzip(filename);
        } else {
            file = ReadFileFormat::text(filename);
        }
        for (i, line) in file.enumerate() {
            if !line.starts_with(args.comment_chars) {
                count += 1;
                if args.skip.contains(&count) {
                    skipped += 1;
                    continue;
                }

                if args.echo {
                    if write_output_check_err(&mut output, &line) {
                        // break if can't write to stdout, (when pipe is closed)
                        break;
                    }
                } else {
                    let row: Vec<String> = line
                        .split(args.input_deliminator)
                        .map(sanitize_cell)
                        .collect();
                    if args.names {
                        for (i, name) in row.iter().enumerate() {
                            println!("{}: {}", i + 1, name);
                        }
                        return;
                    }

                    if col_len.is_none() {
                        col_len = Some(row.len());
                    } else if !(Some(row.len()) == col_len) {
                        if args.ignore_errors {
                            eprintln!("Length Not matched on Line {} (original: {})", count, i);
                            // just ignore that line
                            continue;
                        } else {
                            panic!("Length Not matched");
                        }
                    }

                    if args.filter.is_some() {
                        let mut flag = false;
                        for (k, v) in &filter_vals {
                            if row.get(*k - 1).expect("Key Column Number out of range") == v {
                                flag = true;
                            }
                        }
                        if !flag {
                            continue;
                        }
                    }

                    if let Some(ref columns) = args.columns {
                        output_line = (&args.delineate.to_string()).join(
                            row.iter()
                                .enumerate()
                                .filter(|x| columns.contains(&(x.0 + 1)))
                                .map(|x| x.1),
                        );
                    } else {
                        output_line = row.join(&args.delineate.to_string())
                    }

                    if write_output_check_err(&mut output, &output_line) {
                        break;
                    }
                }

                if Some(count - skipped) == args.head {
                    break;
                }
            }
        }
    }
}
