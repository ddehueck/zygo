import pytest

from zygo import Channel, JobContext, Workflow
from zygo._internal.meta.jobs import validate_job
from zygo.codecs import Integer, String


def test_job_accepts_all_supported_return_annotations() -> None:
    input_channel = Channel(id="input", codec=Integer())
    output_channel = Channel(id="output", codec=String())
    workflow = Workflow(
        id="return-annotations",
        input=input_channel,
        output=output_channel,
    )

    @workflow.job(input=input_channel, output=output_channel)
    def scalar(value: int) -> str:
        return str(value)

    @workflow.job(input=input_channel, output=output_channel)
    def many(value: int, *, ctx: JobContext) -> list[str]:
        del ctx
        return [str(value)]

    @workflow.job(input=input_channel, output=output_channel)
    def optional_scalar(value: int) -> str | None:
        return str(value) if value else None

    @workflow.job(input=input_channel, output=output_channel)
    def optional_many(value: int) -> list[str] | None:
        return [str(value)] if value else None

    @workflow.job(input=input_channel, output=output_channel)
    def scalar_or_many(value: int) -> str | list[str]:
        return str(value) if value else []

    @workflow.job(input=input_channel, output=output_channel)
    def scalar_or_many_or_none(value: int) -> str | list[str] | None:
        return str(value) if value else None

    registered_functions = tuple(entry.job_fn for entry in workflow.jobs)
    assert registered_functions == (
        scalar,
        many,
        optional_scalar,
        optional_many,
        scalar_or_many,
        scalar_or_many_or_none,
    )


def test_job_rejects_list_of_wrong_output_type() -> None:
    def invalid(value: int) -> list[int]:
        return [value]

    with pytest.raises(ValueError, match="return value must be annotated"):
        validate_job(invalid, input_channel_type=int, output_channel_type=str)


def test_job_rejects_union_containing_wrong_output_type() -> None:
    def invalid(value: int) -> str | list[int]:
        return str(value)

    with pytest.raises(ValueError, match="return value must be annotated"):
        validate_job(invalid, input_channel_type=int, output_channel_type=str)


def test_job_rejects_none_without_an_output_type() -> None:
    def invalid(value: int) -> None:
        del value

    with pytest.raises(ValueError, match="return value must be annotated"):
        validate_job(invalid, input_channel_type=int, output_channel_type=str)
