import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import gen_balance  # noqa: E402


def test_generated_tables_are_current():
    assert gen_balance.main(["--check"]) == 0


def test_tables_parse_constructor_arguments():
    text = gen_balance.generate()
    # visual_defaults passes hedgehog_scale as a0 and arrays as later arguments
    assert '("hedgehog_scale", TableVal::Num(0.6666666666666666)),' in text
    assert '("stinky_cone_deg_levels", TableVal::Arr(&[13.0, 18.5, 24.0, 29.0, 33.0])),' in text
    assert '("cave_icon_smooth", TableVal::Num(1.0)),' in text
    assert '("caves_blue_first_x", TableVal::Arr(&[2.38, 3.74])),' in text
    assert '("start_food", 50.0),' in text


def test_value_parsing():
    assert gen_balance.parse_value("gm.array(1, -2.5)", {}, "t") == [1.0, -2.5]
    assert gen_balance.parse_value("a1", {"a1": "7"}, "t") == 7.0
    assert gen_balance.split_args("1, gm.array(2, 3), 4") == ["1", "gm.array(2, 3)", "4"]
