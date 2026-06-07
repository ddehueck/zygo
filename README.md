# Zygo
A modern workflow management system for bioinformatics in Python

```🚨 Zygo is an active work in progress. Do not use it in production.```

Zygo consists of three components:

- A Python library for creating workflows
- A GUI for visualizing and debugging workflows
- A CLI for managing workflows

## Example

```python
from zygo import Workflow, Store, Channel, From, To

workflow = Workflow(name="my_workflow")

reads       = workflow.channel(name="reads")
qc_reports  = workflow.channel(name="qc_reports")
final_files = workflow.channel(name="final")


@workflow.job
def reads_to_qc_reports(
	reads_ref: Annotated[Reference, Input(reads)]
  	qc_report_publisher: Annotated[Publisher, Output(qc_reports)],
  	store: Annotated[Store, Depends(Store)],
):
	reads = store.get(reads_ref)
	qc_reports = do_qc(reads)
	qc_report_publisher.publish(qc_reports)
	
@workflow.job
def qc_reports_to_final(
	qc_reports_ref: Annotated[Reference, Input(qc_reports)],
	final_files_publisher: Annotated[Publisher, Output(final_files)],
	store: Annotated[Store, Depends(Store)],
):
	qc_reports = store.get(qc_reports_ref)
	final_files = do_something(qc_reports)
	final_files_publisher.publish(result)

@workflow.job
def final(
	final_file_ref: Annotated[Reference, Input(final_files)],
	store: Annotated[Store, Depends(Store)],
):
	print(f"Final file: {store.get(final_file_ref)}")

if __name__ == "__main__":
  workflow.run(
		channel=reads,
		uri="file://data.csv",
		backend=LocalBackend(store_uri="./my_data"),
	)

```

### Core Concepts
___

- **Jobs** are the fundamental building blocks of a workflow. They are functions that are executed by the workflow engine.
- **Channels** are the pipes that connect jobs. They are used to pass data references between jobs.
- **Workflows** are the composition of jobs and channels into a useful application.
- **The Store** is the key-value interface where data is accessed and published.
- **The Backend** says where the workflow should run and where the data should live.

### Jobs
___
Jobs are functions that are executed by the workflow engine. They are decorated with the `@workflow.job` decorator. They can listen to only one channel but publish to many channels.

They are considered pure functions meaning that given the same input, the output will be the same. This allows Zygo to easily cache results and re-use them while other parts of the workflow are being developed. If this is not possible for your use case, you set cache=False on the job decorator.

