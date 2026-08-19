import random
import time
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

workflow = Workflow(id="my_workflow")

raw_values = workflow.channel(id="raw_values", is_input=True)
squared_values = workflow.channel(id="squared_values")

workflow.job()


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

    rand_wait = random.randint(1, 15)
    for i in range(rand_wait):
        print(f"[square_values] Waiting: {i + 1}/{rand_wait}")  # noqa: T201
        time.sleep(1)


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
    print(f"[squared_values_to_final] Received: data!")  # noqa: T201
    received = store.get(squared_values)
    received = int.from_bytes(received)

    rand_wait = random.randint(1, 15)
    for i in range(rand_wait):
        print(f"[square_values] Que Tal Waiting: {i + 1}/{rand_wait}")  # noqa: T201
        time.sleep( 1)


    print(f"[squared_values_to_final] Squared received: {received}")  # noqa: T201
