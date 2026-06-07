from pathlib import Path
from typing import Annotated

from zygo import (
    Depends,
    Input,
    Output,
    Publisher,
    Reference,
    Store,
    Workflow,
)
from zygo.backends.default import DefaultBackend
from zygo.types import Environment


HERE = Path(__file__).resolve().parent

workflow = Workflow(name="my_workflow")

raw_values = workflow.channel(name="raw_values")
squared_values = workflow.channel(name="squared_values")


@workflow.job
def square_values(
    input: Annotated[Reference, Input(raw_values)],
    publisher: Annotated[Publisher, Output(squared_values)],
    store: Annotated[Store, Depends(Store)],
) -> None:
    received = store.get(input)
    received = int(received)
    print(f"[reads_to_qc_reports] GOING TO SQUARE: {received}")  # noqa: T201

    squared: int = received * received

    publisher.publish(
        store.put(
            key="squared.txt",
            data=squared.to_bytes(8, byteorder="big"),
            scope="job",
            content_type="text/plain",
        )
    )
    print("Squared value published")  # noqa: T201


@workflow.job
def squared_values_to_final(
    squared_values: Annotated[Reference, Input(squared_values)],
    store: Annotated[Store, Depends(Store)],
) -> None:
    received = store.get(squared_values)
    received = int.from_bytes(received)
    print(f"[squared_values_to_final] Squared received: {received}")  # noqa: T201


if __name__ == "__main__":
    print("Running workflow...")  # noqa: T201

    workflow.run(
        channel=raw_values,
        uri=f"file://{HERE / 'input.txt'}",
        backend=DefaultBackend(store_uri="./zygo"),
    )
