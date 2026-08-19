from collections.abc import Iterator
from dataclasses import dataclass
from types import FunctionType
from typing import Any

from zygo._internal.fn_hash import local_source_dependency_hash
from zygo.channel import Channel
from zygo.types import JobHash, JobId


@dataclass(frozen=True)
class JobEntry:
    id: JobId
    hash: JobHash
    job_fn: FunctionType
    input_channel: Channel[Any]  # pyright: ignore[reportExplicitAny]
    output_channel: Channel[Any]  # pyright: ignore[reportExplicitAny]


class DuplicateJobError(Exception):
    """Raised when attempting to register a job with a duplicate name or hash."""

    pass


class JobRegistry:
    def __init__(self) -> None:
        super().__init__()
        self._jobs_by_id: dict[JobId, JobEntry] = {}

    def get_by_id(self, id: JobId) -> JobEntry | None:
        if id not in self._jobs_by_id:
            return None
        return self._jobs_by_id[id]

    def set(
        self,
        *,
        job: FunctionType,
        input_channel: Channel[Any],  # pyright: ignore[reportExplicitAny]
        output_channel: Channel[Any],  # pyright: ignore[reportExplicitAny]
    ) -> JobEntry:
        """
        Register a job with the given name.

        Raises:
            DuplicateJobError: If a job with the same id already exists.
        """
        job_id = self._name_as_id(job)

        if job_id in self:
            raise DuplicateJobError(f"A job with id '{job_id}' already exists")

        job_hash = local_source_dependency_hash(job).hash_str
        entry = JobEntry(
            id=job_id,
            hash=JobHash(job_hash),
            job_fn=job,
            input_channel=input_channel,
            output_channel=output_channel,
        )

        self._jobs_by_id[job_id] = entry

        return entry

    def __contains__(self, key: JobId) -> bool:
        return key in self._jobs_by_id

    def __len__(self) -> int:
        return len(self._jobs_by_id)

    def __iter__(self) -> Iterator[JobEntry]:
        return iter(self._jobs_by_id.values())

    @staticmethod
    def _name_as_id(job: FunctionType) -> JobId:
        return JobId(job.__name__)
