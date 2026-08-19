# Zygo Architecture

This document describes the architecture of Zygo.

## Properties of a Zygo Workflow

A zygo workflow is made up of jobs and channels. Jobs do work. Channels send data into jobs and receive data from jobs.

### Jobs

A job is a unary function. It has one input and one output which also means it has one input channel and one output channel.

One input doesn't mean one primitive data type like an `int` or `string` the input could be a `dict` or `list` or even a `Folder` or other custom data types.

This keeps dataflow simple. **One thing in and one thing out.**

Example Job:

```python
from zygo import Workflow, Job, Channel
from my_src import Report, build_report

sequences = Channel(str)
reports = Channel(Report

workflow = Workflow(id="my_workflow", input=sequences)

@workflow.job(input=sequences, output=reports)
def my_job(sequence: str) -> Report:
    report = build_report(sequence)
    return report
```

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

### Typing

With python typing we can say that a channel holds a specific type of data. For example, a channel that holds `int` data would be typed `Channel[int]`. This way there is strong IDE support and type checkers like `ty`, `pyright`, `mypy` etc. can catch bugs before they happen.

This means you can also create custom data types and use them with Channels. 

```python
class LabResult(TypedDict):
    name: str
    result: float

results: Channel[LabResult] = ...
```

As a result of leveraging python typing, we can ensure that the data flow is consistent when multiple jobs write to the same channel and while reading from multiple channels.


# Python Syntax
- `add_job()` for imported ordinary functions.
- `@workflow.job()` for local, context-aware jobs.
