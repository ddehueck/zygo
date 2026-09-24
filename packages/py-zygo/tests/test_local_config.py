from pathlib import Path
from types import ModuleType

import pytest

from zygo.cli.v0.config import local_store_options, project_search_paths


def test_local_store_options_precedence_and_fallback(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    project = tmp_path / "project"
    module_dir = project / "workflows"
    module_dir.mkdir(parents=True)
    module = ModuleType("workflows.main")
    module.__file__ = str(module_dir / "main.py")

    (tmp_path / "pyproject.toml").write_text(
        '[tool.zygo.local]\ndata_dir = "outside-project"\n'
    )
    pyproject = project / "pyproject.toml"
    pyproject.write_text('[tool.zygo.local]\ndata_dir = "results"\n')
    monkeypatch.chdir(project)

    assert project_search_paths(module) == [module_dir, project]
    assert local_store_options(module).root_uri.uri == f"file://{project / 'results'}"
    assert local_store_options(module, "file:///override").root_uri.uri == (
        "file:///override"
    )

    pyproject.unlink()
    assert local_store_options(module).root_uri.uri == f"file://{module_dir / 'zygo'}"
