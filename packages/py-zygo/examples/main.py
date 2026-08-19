import random
import time

from zygo import Channel, Workflow
from zygo.codecs import Integer, String

raw = Channel(id="raw", codec=Integer())
squared = Channel(id="squared", codec=Integer())
output = Channel(id="output", codec=String())

workflow = Workflow(id="my_workflow", input=raw, output=output)


@workflow.job(input=raw, output=squared)
def square_values(input: int) -> int:
    print(f"[reads_to_qc_reports] GOING TO SQUARE: {input}")  # noqa: T201

    rand_wait = random.randint(1, 15)
    for i in range(rand_wait):
        print(f"[square_values] Waiting: {i + 1}/{rand_wait}")  # noqa: T201
        time.sleep(1)

    return input * input


@workflow.job(input=squared, output=output)
def last_step(squared_value: int) -> str | None:
    print(f"[last_step] Received: {squared_value}")  # noqa: T201

    rand_wait = random.randint(1, 15)
    for i in range(rand_wait):
        print(f"[last_step] Waiting: {i + 1}/{rand_wait}")  # noqa: T201
        time.sleep(1)

    if rand_wait % 2 == 0:
        print(f"[last_step] Even wait received: {squared_value}. Skipping...")  # noqa: T201
        return None

    print(f"[last_step] Squared received: {squared_value}")  # noqa: T201
    return f"result={squared_value}"
