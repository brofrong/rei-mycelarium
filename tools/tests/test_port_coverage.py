import subprocess
import sys
from pathlib import Path

TOOL = Path(__file__).resolve().parents[1] / "port_coverage.py"


def run(tmp_path, portmap, rust_src, *args):
    pm = tmp_path / "portmap.tsv"
    pm.write_text(portmap)
    src = tmp_path / "src"
    src.mkdir(exist_ok=True)
    (src / "lib.rs").write_text(rust_src)
    return subprocess.run(
        [sys.executable, "-I", str(TOOL), "--portmap", str(pm), "--src", str(src), *args],
        capture_output=True, text=True)


def test_missing_function_reported(tmp_path):
    r = run(tmp_path, "foo\t-\tT5\n", "pub fn foo() {}\n", "--task", "T5")
    assert r.returncode == 1
    assert "foo" in r.stdout


def test_marker_covers(tmp_path):
    r = run(tmp_path, "foo\t-\tT5\n", "// port: foo\npub fn foo() {}\n", "--task", "T5")
    assert r.returncode == 0, r.stdout


def test_alias_marker_covers(tmp_path):
    r = run(tmp_path, "anon__1__x\tmyc_foo\tT8\n", "    // port: myc_foo\nfn f() {}\n", "--task", "T8")
    assert r.returncode == 0, r.stdout


def test_skip_not_required(tmp_path):
    r = run(tmp_path, "bar\t-\tskip:steam\nfoo\t-\tT5\n", "// port: foo\n", "--all")
    assert r.returncode == 0, r.stdout


def test_other_task_not_checked(tmp_path):
    r = run(tmp_path, "foo\t-\tT5\nbaz\t-\tT6\n", "// port: foo\n", "--task", "T5")
    assert r.returncode == 0, r.stdout
    r = run(tmp_path, "foo\t-\tT5\nbaz\t-\tT6\n", "// port: foo\n", "--all")
    assert r.returncode == 1
    assert "baz" in r.stdout


def test_unknown_task_rejected(tmp_path):
    r = run(tmp_path, "foo\t-\tT5\n", "// port: foo\n", "--task", "T55")
    assert r.returncode == 2
    assert "T55" in r.stdout + r.stderr


def test_markers_in_tests_dir_ignored(tmp_path):
    src = tmp_path / "src"
    (src / "crate" / "tests").mkdir(parents=True)
    (src / "crate" / "tests" / "t.rs").write_text("// port: foo\n")
    r = run(tmp_path, "foo\t-\tT5\n", "fn x() {}\n", "--task", "T5")
    assert r.returncode == 1
    assert "foo" in r.stdout
