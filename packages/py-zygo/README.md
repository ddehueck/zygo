# zygo

The package has two responsibilities:

- provide the python interfaces to declare a workflow
- provide a CLI implementing [`protocol/v0`](../../protocol/v0) so orchestrators can inspect workflows and execute jobs

The Python `Store` reads and writes workflow payloads through `fsspec`. The
local orchestrator separately persists run events and read models. The Python
CLI publishes data references through the versioned protocol, which other
orchestrators can implement as well.

To choose where local job results are stored, add this to your workflow project's
`pyproject.toml`:

```toml
[tool.zygo.local]
data_dir = "results"
```

Relative paths are resolved against the directory containing `pyproject.toml`.
The CLI looks for `pyproject.toml` from the imported workflow module's
directory up to the directory where the CLI was invoked, inclusive. It does
not search above that directory. If the module is outside it, the CLI checks
only the working directory. If `data_dir` is not set, results go to a `zygo`
directory beside the workflow module (or in the working directory for modules
outside it). An orchestrator can override this for an individual job with
`root_uri` in the CLI's `--store-config` JSON payload. Optional `kwargs` string pairs are passed to fsspec.
