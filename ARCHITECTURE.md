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
- `Core Crate`: The core logic of the framework. Given a workflow definition, it handles what job should run when with what data.
- `Local Crate`: The local application layer that the CLI and desktop are built on top of.

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

### The Core Crate

The heart of zygo is written in rust in the zygo-core crate. Rust was chosen for it's expressive type system, ecosystem, and performance as zygo may venture into high performance tooling for specific workloads.

This crate defines the core data models and workflow execution engine.

#### The Stream

Each workflow run is an ordered stream of `Event`s and `Command`s - referred to as `StreamItems`.

Each `StreamItem` is linearized via a single `StreamWriter` and written to the stream keyspace with contigous monotonic integer ids. This makes it easy to read the stream and make decisions at a given point in time.

#### The Engine

The engine is designed to process a workflow stream one at a time, applying the workflow rules and executing the associated jobs.

It is spilt into two main components **The Arbiter** and **The Executor**.

The **Arbiter** is responsible for deciding what action to take next based the `Event` history at one point in time. When a decision is made, it issues `Command`s that the executor can then execute.

The **Executor** is responsible for executing `Command`s issued by the arbiter. It may run a job locally, replay events for a cached job, or mark a job as completed.

This is analgous to a standard inbox/outbox pattern where the arbiter reads `Event`s from the inbox and writes `Command`s to the outbox. The executor reads `Command`s from the outbox and executes them - sometimes resulting in more `Event`s being written to the inbox.

In principle, there are optimizations to further parallelize the engine's operations we favor the simplicity of a single-threaded, ordered execution model. 

#### The Actor

todo

### The Local Crate

The local crate is responsible for providing simple application layer for building local tools to interact with the core create. 

For example, the desktop app and CLI tool can both be used to run workflows locally. When running workflows locally, we want to keep a history of workflow runs and provide easy search and filtering capabilities. So, we need a local db to store this information and index it. We don't want this in the core crate because this is a local-only concern.

This way zygo's core logic can be extended to build various applications and services e.g. a cloud zygo service.


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
