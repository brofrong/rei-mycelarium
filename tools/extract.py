"""Entry point: python3 -I tools/extract.py [--game-dir DIR] [--out DIR]"""

import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from myco_extract.extract import main  # noqa: E402

sys.exit(main())
