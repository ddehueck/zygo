"""
A backend is a class that provides a way to deploy and run the jobs of a workflow.

```python
workflow = Workflow(id="my_workflow", backend=MyBackend())
```
"""

from typing import Protocol

from zygo.store.types import StoreOptions


class Backend(Protocol):
    """
    A backend is a class that provides a way to deploy and run the jobs of a workflow.

    ```python
    my_backend = MyBackend(store_uri="s3://my-bucket/my-workflow", api_key="my-api-key")
    ```
    """

    @property
    def allow_local_store(self) -> bool:
        """Whether to allow a local filesystem store to be used.
        e.g. if a local filesystem store is used, this backend will raise an error.
        """
        raise NotImplementedError("Backend's allow_local_store is not implemented.")

    @property
    def store_options(self) -> StoreOptions:
        """The options for the store to use for the workflow."""
        raise NotImplementedError("Backend's store_options is not implemented.")
