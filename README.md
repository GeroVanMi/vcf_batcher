# 📑️ VCF Batcher

[![Rust](https://github.com/GeroVanMi/vcf_batcher/actions/workflows/build.yml/badge.svg)](https://github.com/GeroVanMi/vcf_batcher/actions/workflows/build.yml)

This is a Rust crate to cut VCF (variant call files) into smaller batches, intended to be used for multiprocessing or distributed computing.

## 🧰️ Installation

Depending on what your goals are, you can use this tool as a [CLI](https://en.wikipedia.org/wiki/Command-line_interface) or as a library in
🦀️ Rust or 🐍️ Python.

### Installing the CLI

In order to install the program as a CLI, you will need to have `cargo` installed.
[Instructions to install cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

Once you have it, you can run the following command in your terminal to install the VCF batcher.

```
cargo install vcf_batcher
```

### Installing Rust Crate

In order to install the tool as a rust crate, you can add it to your `Cargo.toml` dependencies or
run:

```
cargo add vcf_batcher
```

You can find the crate documentation on [docs.rs](https://docs.rs/vcf_batcher/latest/vcf_batcher/).

### Installing python bindings

We provide python bindings for the VCF batcher which can be installed via `pip`.

```
pip install vcf-batcher
```

## 🪄️ Usage

### CLI

Using the CLI after installing can be done through the `vcf_batcher_cli` command.

```
vcf_batcher_cli path/to/your_file.vcf path/to/ouput/directory
```

By default, this will create batches with 25'000 samples each. If you'd like to override this
default, you can do so by providing a custom `--batch-size` or `-b` argument:

```
vcf_batcher_cli -b 1000 path/to/your_file.vcf path/to/ouput/directory
```

### Library

After installing either the rust crate or python module, you can use the provided function.

#### 🦀️ Rust

```rust
pub fn extract_variants_to_batches(
    file_path: &str,
    batch_size: usize,
    output_path: &Path,
    compression_level: Option<Compression>
)
```

#### 🐍️ Python

```python
vcf_batcher.py_extract_variants_to_batches(
        input_file,
        batches_folder,
        batch_size,
)
```

## License

The software is licensed under the [MIT License](LICENSE).
