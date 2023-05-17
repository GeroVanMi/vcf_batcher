//! # VCF Batcher
//! A library for converting large VCF files into batches of smaller VCF files containing a fixed number of samples.
//! Can also be used as a command line tool.

use std::fs;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use self::ReaderLines::{UnzippedLines, ZippedLines};
use bgzip::{write::BGZFMultiThreadWriter, BGZFError, BGZFReader, Compression};
use pyo3::prelude::*;

trait AppendLine {
    fn append_line(&mut self, line: &str) -> &String;
}

impl AppendLine for String {
    fn append_line(&mut self, content: &str) -> &String {
        self.push_str(content);
        self.push('\n');
        self
    }
}

/// Wrapper for the lines of a file.
/// If the file is bgzipped, the lines are read with a BGZFReader.
pub enum ReaderLines {
    UnzippedLines(io::Lines<BufReader<File>>),
    ZippedLines(io::Lines<BGZFReader<File>>),
}

impl Iterator for ReaderLines {
    type Item = Result<String, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            UnzippedLines(lines) => lines.next(),
            ZippedLines(lines) => lines.next(),
        }
    }
}

/// Saves a batch of variants to a file.
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use vcf_batcher::save_batch;
/// save_batch("Hello, world!".to_string(), &1, Path::new("test_data/temporary"), None);
/// ```
pub fn save_batch(
    contents: String,
    batch_number: &i32,
    output_path: &Path,
    compression_level: Option<Compression>,
) -> Result<(), BGZFError> {
    fs::create_dir_all(output_path).expect("An error occurred creating the directory");

    let mut file_name = format!("batch_{:02}.vcf", batch_number);

    if let Some(..) = compression_level {
        file_name.push_str(".gz");
        let vcf_path = output_path.join(file_name);

        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = File::create(vcf_path)?;

        let mut write_buffer = Vec::new();
        let mut writer = BGZFMultiThreadWriter::new(&mut write_buffer, compression_level.unwrap());
        writer.write_all(contents.as_bytes())?;
        writer.close()?;

        // Write the content string to `file`, returns `io::Result<()>`
        file.write_all(&write_buffer)?;
    } else {
        let vcf_path = output_path.join(file_name);
        // Open a file in write-only mode, returns `io::Result<File>`
        let mut file = File::create(vcf_path)?;
        file.write_all(contents.as_bytes())?;
    }

    Ok(())
}

/// The output is wrapped in a Result to allow matching on errors
/// Returns an Iterator to the Reader of the lines of the file.
pub fn read_lines<P>(file_path: P) -> Result<ReaderLines, io::Error>
where
    P: AsRef<Path>,
{
    let file = File::open(&file_path).expect("File does not exist.");
    // If the file ends in .gz, we assume it is bgzipped
    if file_path.as_ref().to_str().unwrap().ends_with(".gz") {
        let reader = BGZFReader::new(file).expect("An error occurred reading the compressed file.");
        return Ok(ZippedLines(reader.lines()));
    }

    Ok(UnzippedLines(BufReader::new(file).lines()))
}

/// In VCF-Files header lines containing metadata start with a `#`.
/// This function therefore simply checks if a line starts with a `#`.
///
/// # Examples
///
/// ```
///
/// use vcf_batcher::{is_header_line, read_lines};
///
/// let file_path = "test_data/batch_01.vcf.gz";
/// let mut headers = String::new();
///
/// trait AppendLine {
///     fn append_line(&mut self, line: &str) -> &String;
/// }
///
/// impl AppendLine for String {
///     fn append_line(&mut self, content: &str) -> &String {
///         self.push_str(content);
///         self.push('\n');
///         self
///     }
/// }
///
/// if let Ok(lines) = read_lines(file_path) {
///     for line in lines.flatten() {
///         if is_header_line(&line) {
///             headers.append_line(&line);
///             continue;
///         }
///     }
/// }
/// ```
pub fn is_header_line(line: &str) -> bool {
    line.starts_with('#')
}

/// Converts a large VCF file into batches of smaller VCF files containing a fixed number of samples
pub fn extract_variants_to_batches(
    file_path: &str,
    batch_size: usize,
    output_path: &Path,
    compression_level: Option<Compression>,
) {
    let mut current_batch = String::new();
    let mut headers = String::new();

    // File hosts must exist in current path before this produces output
    let mut current_batch_counter = 0;
    let mut batch_count = 0;

    if let Ok(lines) = read_lines(file_path) {
        // Consumes the iterator, returns an (Optional) String
        for line in lines.flatten() {
            if is_header_line(&line) {
                headers.append_line(&line);
                continue;
            }

            current_batch_counter += 1;
            current_batch.append_line(&line);

            if current_batch_counter >= batch_size {
                batch_count += 1;
                if let Err(error) = save_batch(
                    headers.to_owned() + &current_batch,
                    &batch_count,
                    output_path,
                    compression_level,
                ) {
                    panic!(
                        "An error occurred while trying to save batch {}: {}",
                        batch_count, error
                    )
                }

                if compression_level.is_some() {
                    println!("Saving batch_{:02}.vcf.gz", batch_count);
                } else {
                    println!("Saving batch_{:02}.vcf", batch_count);
                }

                current_batch = String::new();
                current_batch_counter = 0;
            }
        }

        if !current_batch.is_empty() {
            batch_count += 1;
            println!(
                "Saving final batch with less than {} samples to batch_{:02}.vcf.gz",
                batch_size, batch_count
            );

            if let Err(error) = save_batch(
                headers.to_owned() + &current_batch,
                &batch_count,
                output_path,
                compression_level,
            ) {
                panic!(
                    "An error occurred while trying to save batch {}: {}",
                    batch_count, error
                )
            }
        }
        println!(
            "Saved {} batches with {} samples to {}.",
            batch_count,
            batch_size,
            output_path.display()
        );
    } else {
        panic!("An error occurred while trying to read the file. Does it exist and is it either a .vcf or .vcf.gz file?")
    }
}

/// Parses the user input for the compression level and returns the corresponding compression level
/// from the bgzip crate.
pub fn parse_compression_level(raw_compression_level: Option<String>) -> Option<Compression> {
    match raw_compression_level {
        Some(user_input) => match user_input.to_lowercase().as_ref() {
            "fast" => Some(Compression::fast()),
            "best" => Some(Compression::best()),
            "default" => Some(Compression::default()),
            _ => None,
        },
        None => None,
    }
}

/// Wrapper function for extract_variants_to_batches to be called from Python
#[pyfunction]
fn py_extract_variants_to_batches(
    file_path: &str,
    output_path: &str,
    batch_size: usize,
    compression_level: Option<String>,
) -> PyResult<()> {
    extract_variants_to_batches(
        file_path,
        batch_size,
        Path::new(output_path),
        parse_compression_level(compression_level),
    );
    Ok(())
}

#[pymodule]
fn vcf_batcher(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(py_extract_variants_to_batches, m)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use bgzip::Compression;

    use crate::{extract_variants_to_batches, is_header_line, parse_compression_level, read_lines};

    #[test]
    fn test_is_header_line() {
        assert_eq!(
            is_header_line(
                "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tNA00001\tNA00002\tNA00003"
            ),
            true
        );
    }

    #[test]
    fn test_is_not_header_line() {
        assert_eq!(
            is_header_line("1\t1000\t.\tA\tG\t100\tPASS\t.\tGT\t0|0\t0|0\t0|0"),
            false
        );
    }

    #[test]
    fn test_extract_variants_to_batches() {
        let file_path = "./test_data/batch_01.vcf.gz";
        let compression_level = None;
        extract_variants_to_batches(
            file_path,
            10,
            std::path::Path::new("./test_data/result_batches"),
            compression_level,
        );
        // Check if 10 batches were created
        for i in 1..=10 {
            let batch_file_path = match compression_level {
                Some(_) => format!("./test_data/result_batches/batch_{:02}.vcf.gz", i),
                _ => format!("./test_data/result_batches/batch_{:02}.vcf", i),
            };
            if let Ok(mut lines) = read_lines(batch_file_path.clone()) {
                // Check if the first 30 lines of the first file are header lines
                for i in 1..=30 {
                    if let Some(Ok(line)) = lines.next() {
                        assert_eq!(is_header_line(&line), true);
                    } else {
                        panic!("Could not read line {}", i);
                    }
                }
                // Check if the next 10 lines exist
                for i in 1..=10 {
                    if let Some(Ok(line)) = lines.next() {
                        assert_eq!(is_header_line(&line), false);
                    } else {
                        panic!("Could not read line {}", i);
                    }
                }
            } else {
                panic!("Could not read file {}", batch_file_path);
            }
        }
    }

    #[test]
    fn test_parse_compression() {
        assert_eq!(
            parse_compression_level(Some("fast".to_string())),
            Some(Compression::fast())
        );
        assert_eq!(
            parse_compression_level(Some("best".to_string())),
            Some(Compression::best())
        );
        assert_eq!(
            parse_compression_level(Some("default".to_string())),
            Some(Compression::default())
        );
        assert_eq!(parse_compression_level(Some("none".to_string())), None);
        assert_eq!(parse_compression_level(Some("invalid".to_string())), None);
        assert_eq!(parse_compression_level(None), None);
    }
}
