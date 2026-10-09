from io import BytesIO
from pathlib import Path
from zipfile import ZipFile

from zygo.cloud.train.source import SourcePackage


def test_package_skips_tool_directories(tmp_path: Path) -> None:
    (tmp_path / "Dockerfile").write_text("FROM scratch\n")
    (tmp_path / "app.py").write_text("print(1)\n")
    cache = tmp_path / "__pycache__"
    cache.mkdir()
    (cache / "app.pyc").write_bytes(b"pyc")
    git = tmp_path / ".git"
    git.mkdir()
    (git / "config").write_text("git\n")

    package = SourcePackage(tmp_path)
    names = _names(package.archive())

    assert names == ["Dockerfile", "app.py"]
    assert package.excluded() == [".git/", "__pycache__/"]


def test_package_honors_gitignore_and_extra_excludes(tmp_path: Path) -> None:
    (tmp_path / "Dockerfile").write_text("FROM scratch\n")
    (tmp_path / "app.py").write_text("print(1)\n")
    (tmp_path / "notes.log").write_text("log\n")
    (tmp_path / "keep.log").write_text("keep\n")
    (tmp_path / "secret.pem").write_text("pem\n")
    (tmp_path / ".gitignore").write_text("*.log\n!keep.log\nbuild/\n")
    build = tmp_path / "build"
    build.mkdir()
    (build / "output.bin").write_bytes(b"bin")
    nested = tmp_path / "pkg"
    nested.mkdir()
    (nested / ".gitignore").write_text("hidden.py\n")
    (nested / "hidden.py").write_text("print(2)\n")
    (nested / "public.py").write_text("print(3)\n")

    package = SourcePackage(tmp_path, excludes=("*.pem",))
    names = _names(package.archive())

    assert names == [
        ".gitignore",
        "Dockerfile",
        "app.py",
        "keep.log",
        "pkg/.gitignore",
        "pkg/public.py",
    ]
    assert package.excluded() == ["build/", "notes.log", "pkg/hidden.py", "secret.pem"]


def test_package_can_replace_filters(tmp_path: Path) -> None:
    (tmp_path / "Dockerfile").write_text("FROM scratch\n")
    (tmp_path / "notes.md").write_text("notes\n")
    (tmp_path / "app.py").write_text("print(1)\n")

    class DropMarkdown:
        def __init__(self) -> None:
            super().__init__()
            self.suffix = ".md"

        def excludes(self, relative: str) -> bool:
            return relative.endswith(self.suffix)

    names = _names(SourcePackage(tmp_path, filters=(DropMarkdown(),)).archive())

    assert names == ["Dockerfile", "app.py"]


def _names(archive: bytes) -> list[str]:
    with ZipFile(BytesIO(archive)) as packed:
        return packed.namelist()
