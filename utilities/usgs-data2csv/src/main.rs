use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use string_join::Join;

#[derive(Parser)]
#[clap(about = "Converts the tab sep data from usgs website to csv format.")]
struct Cli {
    /// Comment Character
    #[clap(short, long, default_value = "#")]
    comment_chars: char,
    /// Delineate Character
    #[clap(short, long, default_value = ",")]
    delineate: char,
    /// Echo Contents (Skips comments)
    #[clap(short, long)]
    echo: bool,
    /// Ignore Errors
    #[clap(short, long)]
    ignore_errors: bool,
    /// TODO Output file
    #[clap(short, long)]
    output: Option<PathBuf>,
    /// Number of head lines to print
    #[clap(short, long)]
    head: Option<u64>,
    /// Specific line numbers to skip (Comments not counted)
    #[clap(short, long, value_delimiter = ',', value_name = "N")]
    skip: Vec<u64>,
    /// Only output these column numbers
    #[clap(short = 'C', long, value_delimiter = ',', value_name = "N")]
    columns: Option<Vec<usize>>,
    /// Split by this Column and write into separate files, requires output
    #[clap(short = 'S', long, requires = "output", value_name = "N")]
    split: Option<usize>,
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
        Some(r) => writeln!(r, "{}", line).is_err(),
        None => writeln!(std::io::stdout(), "{}", line).is_err(),
    }
}

fn main() {
    let args: Cli = Cli::parse();
    let mut file: File;
    let mut reader;
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

    let mut output_line;

    for filename in args.files {
        if !filename.is_file() {
            continue;
        }
        file = File::open(&filename).unwrap();
        reader = BufReader::new(file);

        for (i, line) in reader.lines().enumerate() {
            let line = line.unwrap();
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
                    let row: Vec<String> = line.split("\t").map(sanitize_cell).collect();
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
