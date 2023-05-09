use bgzip::Compression;
use std::path::Path;
use std::time::Instant;

use clap::Parser;
use vcf_batcher_lib::{extract_variants_to_batches, parse_compression_level};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// Clap command line interface for the vcf_batcher crate
/// Possible parameters are:
/// - input_path: The path to the file to read
/// - output_path: The path to the directory to write
/// - (-b, --batch_size): How many lines of data should be contained in the file, excluding the header
/// - (-c, --compression_level): BGzip compression level, options are "Default", Fast", "Best" and "None".
struct Cli {
    /// The path to the file to read
    input_path: String,

    /// The path to the directory to write
    output_path: String,

    /// How many lines of data should be contained in the file, excluding the header
    #[arg(short, long, default_value_t = 25000)]
    batch_size: usize,

    /// BGzip compression level, options are "Default", Fast", "Best" and "None".
    #[arg(short, long)]
    compression_level: Option<String>,
}

fn main() {
    let start = Instant::now();
    let args = Cli::parse();

    let input_path = args.input_path;
    let output_path = Path::new(&args.output_path);
    let batch_size = args.batch_size;

    let compression_level: Option<Compression> = parse_compression_level(args.compression_level);

    extract_variants_to_batches(&input_path, batch_size, output_path, compression_level);

    let elapsed_time = start.elapsed();
    println!(
        "Extracted variants into batches of size {} in: {} seconds",
        batch_size,
        elapsed_time.as_secs_f32()
    );
}
