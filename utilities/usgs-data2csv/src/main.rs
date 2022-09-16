use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

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
    #[clap(short, long)]
    skip: Vec<u64>,
    /// TODO Recursively parse directories, otherwise skip directories
    #[clap(short, long)]
    recursive: bool,
    /// File list to be parsed
    #[clap(required = true)]
    files: Vec<PathBuf>,
}

fn sanitize_cell(raw:&str) -> String {
    let cell = raw.trim();
    if cell.contains(",") {
	return format!("\"{}\"", cell);
    } else{
	return cell.to_string();
    }
}

fn main() {
    let args: Cli = Cli::parse();
    let mut file: File;
    let mut reader;
    let mut count = 0u64;
    let mut skipped = 0u64;
    let mut col_len:Option<usize> = None;

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
                    println!("{}", line);
                } else {
                    let row: Vec<String> = line.split("\t").map(sanitize_cell).collect();
		    if col_len.is_none(){
			col_len = Some(row.len());
		    } else if !(Some(row.len()) == col_len){
			if args.ignore_errors {
			    eprintln!("Length Not matched on Line {} (original: {})", count, i);
			    continue;
			}else{
			    panic!("Length Not matched");
			}
		    }
                    println!("{}", row.join(&args.delineate.to_string()))
                }

                if Some(count - skipped) == args.head {
                    break;
                }
            }
        }
    }
}
