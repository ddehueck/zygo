from pathlib import Path

from zygo.cloud.datasets.editor import DatasetEditor


def test_put_dir_applies_gitignore_patterns_at_any_depth(tmp_path: Path):
    source = tmp_path / "source"
    (source / "nested" / "cache").mkdir(parents=True)
    (source / "cache").mkdir()
    (source / "keep.txt").write_text("keep")
    (source / "keep.tmp").write_text("re-included")
    (source / ".DS_Store").write_text("ignored filename")
    (source / "nested" / "keep.txt").write_text("keep")
    (source / "nested" / "keep.tmp").write_text("re-included")
    (source / "nested" / ".DS_Store").write_text("ignored filename")
    (source / "cache" / "cached.txt").write_text("ignored directory")
    (source / "nested" / "cache" / "cached.txt").write_text("ignored directory")
    (source / "nested" / "cache" / "keep.tmp").write_text(
        "still ignored because parent directory is ignored"
    )

    with DatasetEditor() as editor:
        editor.put_dir(
            source,
            dest="dataset",
            ignore=["*.DS_Store", "cache/", "*.tmp", "!keep.tmp"],
        )

    assert [operation.path for operation in editor.operations] == [
        "dataset/keep.tmp",
        "dataset/keep.txt",
        "dataset/nested/keep.tmp",
        "dataset/nested/keep.txt",
    ]
