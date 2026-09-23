# zygo

The package has two responsibilities:

- provide the python interfaces to declare a workflow
- provide a CLI implementing [`protocol/v0`](../../protocol/v0) so orchestrators can inspect workflows and execute jobs

The Python `Store` reads and writes workflow payloads through `fsspec`. The
local orchestrator separately persists run events and read models. The Python
CLI publishes data references through the versioned protocol, which other
orchestrators can implement as well.
