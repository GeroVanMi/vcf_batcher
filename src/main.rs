use bgzip::Compression;
use std::path::Path;
use std::time::Instant;

use clap::Parser;

mod batch;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    input_path: String,
    /// The path to the directory to write
    output_path: String,
    #[arg(short, long, default_value_t = 100)]
    batch_size: usize,
    // Optional compression level
    #[arg(short, long, default_value = "None")]
    compression_level: String,
}

fn main() {
    let start = Instant::now();
    let args = Cli::parse();

    let input_path = args.input_path;
    let output_path = Path::new(&args.output_path);
    let batch_size = args.batch_size;

    // TODO: Add support for compression level
    let compression_level: Option<Compression> = match args.compression_level {
        _ => None,
    };

    batch::extract_variants_to_batches(
        &input_path,
        batch_size,
        output_path,
        compression_level,
    );

    let elapsed_time = start.elapsed();
    println!(
        "Extracted variants into batches of size {} in: {} seconds",
        batch_size,
        elapsed_time.as_secs_f32()
    );
}
