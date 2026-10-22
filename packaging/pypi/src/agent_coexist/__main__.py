"""Allow `python -m agent_coexist` alongside the console script."""

from . import main

raise SystemExit(main())
