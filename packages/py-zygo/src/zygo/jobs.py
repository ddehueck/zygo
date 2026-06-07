from collections.abc import Iterator
from dataclasses import dataclass
from types import FunctionType

from zygo._internal.python.fn_hash import local_source_dependency_hash
from zygo.backends.protocol import JobConfig
from zygo.types import Environment, JobFnName, JobHash, JobName


@dataclass(frozen=True)
class JobEntry:
    name: JobFnName
    hash: JobHash
    job_fn: FunctionType


class DuplicateJobError(Exception):
    """Raised when attempting to register a job with a duplicate name or hash."""

    pass


class JobRegistry:
    def __init__(self) -> None:
        super().__init__()
        self._jobs_by_name: dict[JobFnName, FunctionType] = {}
        self._jobs_by_hash: dict[JobHash, FunctionType] = {}
        self._environments_by_job_name: dict[JobFnName, Environment] = {}

    def get_by_name(self, name: JobFnName) -> FunctionType | None:
        if name not in self._jobs_by_name:
            return None
        return self._jobs_by_name[name]

    def get_by_hash(self, hash: JobHash) -> FunctionType | None:
        if hash not in self._jobs_by_hash:
            return None
        return self._jobs_by_hash[hash]

    def set(self, job: FunctionType, env: Environment | None = None) -> JobHash:
        """
        Register a job with the given name.

        Raises:
            DuplicateJobError: If a job with the same name or hash already exists.
        """
        job_name = self._name(job)
        job_hash = local_source_dependency_hash(job).hash_str

        if JobHash(job_hash) in self:
            raise DuplicateJobError(f"A job with hash '{job_hash}' already exists")

        # Register the job
        self._jobs_by_name[JobFnName(job_name)] = job
        self._jobs_by_hash[JobHash(job_hash)] = job
        if env is not None:
            self._environments_by_job_name[JobFnName(job_name)] = env
        return JobHash(job_hash)

    def job_configs(self) -> list[JobConfig]:
        return [
            JobConfig(
                id=JobName(entry.name),
                environment=self._environments_by_job_name[entry.name],
            )
            for entry in self.entries()
            if entry.name in self._environments_by_job_name
        ]

    def entries(self) -> list[JobEntry]:
        entries: list[JobEntry] = []
        for name, func in self._jobs_by_name.items():
            hash = None
            for h, f in self._jobs_by_hash.items():
                if f is func:
                    hash = h
                    break
            if hash is not None:
                entries.append(JobEntry(name, hash, func))
        return entries

    def __contains__(self, key: JobName | JobHash | FunctionType) -> bool:
        """Check if a job exists by name or hash."""
        if isinstance(key, FunctionType):
            return self._name(key) in self._jobs_by_name
        return key in self._jobs_by_name or key in self._jobs_by_hash

    def __len__(self) -> int:
        return len(self._jobs_by_name)

    def __iter__(self) -> Iterator[FunctionType]:
        return iter(self._jobs_by_name.values())

    @staticmethod
    def _name(job: FunctionType) -> JobName:
        return JobName(job.__name__)
