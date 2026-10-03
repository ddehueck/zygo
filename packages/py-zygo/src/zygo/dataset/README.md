# Zygo Dataset

This package provides dataset management.

Zygo Datasets are built on top of pyarrow and are loaded via fsspec. This allows for efficient data access in local and remote environments.

These datasets can be used in workflows and act as the input to a train function for ml models.


## Future Work

- [ ] Add streaming support
- [ ] Integrate biological data format semantics like FASTQ, BAM, and SAM files via pyoxbow
