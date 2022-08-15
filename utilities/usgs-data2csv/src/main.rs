use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Parser)]
#[clap(about = "Converts the fixed length data from usgs website to csv format")]
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
    /// Output file
    #[clap(short, long)]
    output: Option<PathBuf>,
    /// Number of head lines to print
    #[clap(short, long)]
    head: Option<u64>,
    /// Specific line numbers to skip (Comments not counted)
    #[clap(short, long)]
    skip: Vec<u64>,
    /// Recursively parse directories, otherwise skip directories
    #[clap(short, long)]
    recursive: bool,
    /// File list to be parsed
    #[clap(required = true)]
    files: Vec<PathBuf>,
}


fn main() {
    let args:Cli = Cli::parse();
    let mut file:File;
    let mut reader;
    let mut count = 0u64;
    let mut skipped = 0u64;
    
    for filename in args.files {
	file = File::open(&filename).unwrap();
	reader = BufReader::new(file);
	
	for line in reader.lines() {
            let line = line.unwrap();
	    if !line.starts_with(args.comment_chars){
		count += 1;
		if args.skip.contains(&count) {
		    skipped += 1;
		    continue;
		}
		
		if args.echo {
		    println!("{}", line);
		} else {
		    let row:Vec<&str> = line.split("\t").map(|x| x.trim()).collect();
		    println!("{}", row.join(&args.delineate.to_string()))
		}

		if Some(count - skipped) == args.head {
		    break;
		}
	    }
    }
    }
}
