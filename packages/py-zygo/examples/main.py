import random
import time

from zygo import Channel, JobContext, Workflow
from zygo.codecs import Integer, String

raw = Channel(id="raw", codec=Integer())
squared = Channel(id="squared", codec=Integer())
output = Channel(id="output", codec=String())

workflow = Workflow(id="my_workflow", input=raw, output=output)


@workflow.job(input=raw, output=squared)
def square_values(input: int, *, ctx: JobContext) -> int:
    tags, store = ctx.tags, ctx.store

    print(f"[reads_to_qc_reports] GOING TO SQUARE: {input}")  # ruff: ignore[print]

    rand_wait = random.randint(1, 15)
    if rand_wait % 2 == 0:
        tags.add("wait_type", "even")
    else:
        tags.add("wait_type", "odd")

    for i in range(rand_wait):
        print(f"[square_values] Waiting: {i + 1}/{rand_wait}")  # ruff: ignore[print]
        time.sleep(1)

    return input * input


@workflow.job(input=squared, output=output)
def last_step(squared_value: int) -> str | None:
    print(f"[last_step] Received: {squared_value}")  # ruff: ignore[print]

    rand_wait = random.randint(1, 15)
    for i in range(rand_wait):
        print(f"[last_step] Waiting: {i + 1}/{rand_wait}")  # ruff: ignore[print]
        time.sleep(1)

    if rand_wait % 2 == 0:
        print(f"[last_step] Even wait received: {squared_value}. Skipping...")  # ruff: ignore[print]
        return None

    print(f"[last_step] Squared received: {squared_value}")  # ruff: ignore[print]
    return f"result={squared_value}"
