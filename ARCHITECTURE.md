# Zygo Architecture

This document describes the architecture of Zygo.

## Properties of a Zygo Workflow

A zygo workflow is made up of jobs and channels. Jobs do work. Channels send data into jobs and receive data from jobs.

### Jobs

A job is a unary function. It has one input and one output which also means it has one input channel and one output channel.

One input doesn't mean one primitive data type like an `int` or `string` the input could be a `dict` or `list` or even a `Folder` or other custom data types.

This keeps dataflow simple. **One thing in and one thing out.**

Example Transformations:
```
pandas.DataFrame    → int
DNASequence         → float
Path                → QCReport
list[Read]          → Alignment
Sample              → VariantReport
dict[str, Sequence] → ConsensusSequence
```

### Channels

Channels are the pipes that connect jobs together. Every piece of data goes into a channel is recieved by every job that listens to that channel.

### Data References

Channels are not where the data is actually stored. You decide where your data is stored i.e. in a local file, cloud storage, or a database. Instead, channels just hold references to the data and the workflow job will only fetch the data when it needs to.

This means you there is no limit to the size of the data you can pass between jobs - its up to you and your storage provider.

## The Pieces

- `Python Library`: The interface developers use to define workflows and jobs.
- `Protocol`: The versioned CLI contract implemented by the Python library and consumed by orchestrators.
- `Local Crate`: The Rust implementation used by the CLI and desktop.

### The Python Library

The Python library is the interface developers use to define workflows and jobs. It aims to be pythonic, composible, type-safe, and easy to use.

Example Workflow Definition:

```python
from zygo import Workflow, Job, Channel
from my_src import Report, build_report

sequences = Channel(id="sequences", type=str)
reports = Channel(id="reports", type=Report)

workflow = Workflow(
    id="my_workflow",
    input=sequences,
    output=reports,
)

@workflow.job(input=sequences, output=reports)
def my_job(sequence: str) -> Report:
    report = build_report(sequence)
    return report
```

#### Composition

Building new workflows on top of existing ones should be easy with zygo. Because every job and workflow has a well-defined input and ouput channel, new workflows can be built by connecting existing channels together.

This means workflows can easily be defined in one python file, then imported as a module into other workflows. All with native python support.


#### Typing

The zygo python package aims to be type-safe without forcing users to use typing (it is recommend though 🙂). 

With python typing we can assign a type to a channel. For example, a channel that holds `int` data would be typed `Channel[int]`. This way there is strong IDE support and type checkers like `ty`, `pyright`, `mypy` etc. can catch bugs before they happen.

This means you can also create custom data types and use them with Channels. 

```python
class LabResult(TypedDict):
    name: str
    result: float

results = Channel(id="results", type=LabResult)
```

### The Protocol

The Python CLI implements the versioned job-provider protocol in [`protocol/v0`](protocol/v0). Orchestrators can use this contract without depending on the local Rust crate.

### The Local Crate

The `local` crate contains the Rust workflow models, Python CLI adapter, and one implementation of the workflow orchestrator used by the desktop app and CLI.




# WIPs

## Data Isolation

Sometimes data needs to be shared across runs/jobs. e.g. ckpt files
Sometimes data will need to be cached globally. e.g. external datasets, model files, etc 
Sometimes a python package won't be fsspec compatible - e.g. 

```python
from mne.datasets import eegbci

paths = eegbci.load_data(
        subjects=subject,
        runs=[4, 8, 12],
        path=download_root,
        update_path=False,
    )
```
