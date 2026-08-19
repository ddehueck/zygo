from collections.abc import Iterator
from dataclasses import dataclass
from types import FunctionType

from zygo._internal.fn_hash import local_source_dependency_hash
from zygo.types import JobHash, JobId


@dataclass(frozen=True)
class JobEntry:
    id: JobId
    hash: JobHash
    job_fn: FunctionType


class DuplicateJobError(Exception):
    """Raised when attempting to register a job with a duplicate name or hash."""

    pass


class JobRegistry:
    def __init__(self) -> None:
        super().__init__()
        self._jobs_by_id: dict[JobId, FunctionType] = {}
        self._jobs_by_hash: dict[JobHash, FunctionType] = {}

    def get_by_id(self, id: JobId) -> FunctionType | None:
        if id not in self._jobs_by_id:
            return None
        return self._jobs_by_id[id]

    def get_by_hash(self, hash: JobHash) -> FunctionType | None:
        if hash not in self._jobs_by_hash:
            return None
        return self._jobs_by_hash[hash]

    def set(self, job: FunctionType) -> JobHash:
        """
        Register a job with the given name.

        Raises:
            DuplicateJobError: If a job with the same name or hash already exists.
        """
        job_hash = local_source_dependency_hash(job).hash_str

        if JobHash(job_hash) in self:
            raise DuplicateJobError(f"A job with hash '{job_hash}' already exists")

        # Register the job
        job_id = self._name_as_id(job)
        self._jobs_by_id[job_id] = job
        self._jobs_by_hash[JobHash(job_hash)] = job
        return JobHash(job_hash)

    def entries(self) -> list[JobEntry]:
        entries: list[JobEntry] = []
        for id, func in self._jobs_by_id.items():
            hash = None
            for h, f in self._jobs_by_hash.items():
                if f is func:
                    hash = h
                    break
            if hash is not None:
                entries.append(JobEntry(id, hash, func))
        return entries

    def __contains__(self, key: JobId | JobHash) -> bool:
        """Check if a job exists by name or hash."""
        return key in self._jobs_by_id or key in self._jobs_by_hash

    def __len__(self) -> int:
        return len(self._jobs_by_id)

    def __iter__(self) -> Iterator[FunctionType]:
        return iter(self._jobs_by_id.values())

    @staticmethod
    def _name_as_id(job: FunctionType) -> JobId:
        return JobId(job.__name__)
