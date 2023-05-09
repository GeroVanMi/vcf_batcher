def py_extract_variants_to_batches(
        file_path: str,
        output_path: str,
        batch_size: int,
        compression_level: str | None = None,
) -> None:
    """
    Converts a large VCF file into batches of smaller VCF files containing a fixed number of samples.

    :param file_path: The VCF file to split into batches.
    :param output_path: The directory where the batches will be saved.
    :param batch_size: The number of samples to include in each batch.
    :param compression_level: The compression level to use when writing the batches. Options are "Default", "Fast", and "Best".
    :return: None
    """
