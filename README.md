# Zygo
A modern workflow management system for bioinformatics in Python

```🚨 Zygo is an active work in progress. Do not use it in production.```

Zygo consists of three components:

- A Python library for creating workflows
- A GUI for visualizing and debugging workflows
- A CLI for managing workflows

## Example

```python
from my_src import QcReport, QcReportCodec
from zygo import Channel, Workflow
from zygo.codecs import Bytes, String
from zygo.context import JobContext


reads = Channel(id="reads", codec=Bytes)
qc_reports = Channel(id="qc_reports", codec=QcReportCodec)
final_files = Channel(id="final_files", codec=String)

workflow = Workflow(
    id="my_workflow",
    input=reads,
    output=final_files,
)


@workflow.job(input=reads, output=qc_reports)
def reads_to_qc_reports(reads_value: bytes, *, ctx: JobContext) -> QcReport:
    return do_qc(reads_value)


@workflow.job(input=qc_reports, output=final_files)
def qc_reports_to_final(qc_report: QcReport) -> str:
    return do_something(qc_report)
```

Zygo allows for workflow composition by default. Intermediate channels can be used to create new workflows by branching off of existing ones.

### Core Concepts
___

- **Jobs** are the fundamental building blocks of a workflow. They are functions that are executed by the workflow engine.
- **Channels** are typed connections between jobs. Each channel codec encodes values for storage and decodes them before job execution.
- **Workflows** are the composition of jobs and channels into a useful application.
- **The Store** is a key-value interface available through `JobContext` for job-specific artifacts and data.
- **The Backend** says where the workflow should run and where the data should live.

### Jobs
___
Jobs are unary functions executed by the workflow engine. Each job consumes one decoded value from its input channel and returns a value for its output channel. Jobs may optionally declare a keyword-only `ctx: JobContext` parameter.

They are considered pure functions meaning that given the same input, the output will be the same. This allows Zygo to easily cache results and re-use them while other parts of the workflow are being developed. If this is not possible for your use case, you set cache=False on the job decorator.
