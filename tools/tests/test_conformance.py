import conformance as c


def test_diff_reports_field_path():
    love = {"state": {"food": 10, "tasks": {"done": ["a"]}}, "mycelium": {"knots": [{"x": 0, "y": 480}]}}
    rust = {"state": {"food": 12, "tasks": {"done": ["a", "b"]}}, "mycelium": {"knots": [{"x": 0.4, "y": 481}]}}
    diffs = c.compare_dumps(love, rust)
    paths = [p for p, _, _ in diffs]
    assert "state.food" in paths
    assert "state.tasks.done.length" in paths
    # positions within 0.5 px match; 1 px does not
    assert "mycelium.knots[0].x" not in paths
    assert "mycelium.knots[0].y" in paths


def test_diff_missing_keys_and_types():
    d = c.diff({"a": 1, "b": {"c": True}}, {"a": 1, "b": {"c": 1}, "z": 0})
    assert ("z", c._MISSING, 0) in d
    assert ("b.c", True, 1) in d


def test_parse_scenario():
    sc = c.parse_scenario("seed: 7\ndump: 900  # end\ninput: 1:gmove:960,600; 61:down:1\ninput: 62:up:1\n", "x")
    assert (sc.seed, sc.dump) == (7, 900)
    assert sc.input_for("love") == "1:gmove:960,600;61:down:1;62:up:1"
    sc = c.parse_scenario("dump: 5\ninput: 1:gmove:1,1; 9:up:1\ninput.rust: 5:down:2\ninput.love: 5:kdown:lalt\n")
    assert sc.input_for("rust") == "1:gmove:1,1;5:down:2;9:up:1"
    assert sc.input_for("love") == "1:gmove:1,1;5:kdown:lalt;9:up:1"


def test_scenarios_parse():
    for p in c.SCENARIOS.glob("*.txt"):
        sc = c.load_scenario(p.stem)
        assert sc.dump > 0, p
