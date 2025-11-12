# Python Smoke Test

Runs `examples/python-smoke/main.py` against the published wheel and against a
local build of `packages/python`.

## 1. Test the latest PyPI release

```bash
cd examples/python-smoke
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install -r requirements.txt
python main.py
```

## 2. Test the local PyO3 bindings

```bash
cd examples/python-smoke
python3 -m venv .venv
source .venv/bin/activate
pip install --upgrade pip
pip install ../../packages/python
python main.py
```

`pip install ../../packages/python` builds the wheel from the current working
copy (ensure Rust is installed). Deactivate or remove `.venv` when done.
