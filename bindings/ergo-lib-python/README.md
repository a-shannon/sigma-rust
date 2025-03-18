[![PyPI version](https://badge.fury.io/py/ergo-lib-python.svg)](https://badge.fury.io/py/ergo-lib-python)
[![Documentation](https://readthedocs.org/projects/ergo-lib-python/badge/?version=latest&style=flat)](https://ergo-lib-python.readthedocs.io)

Python bindings for [sigma-rust](https://github.com/ergoplatform/sigma-rust)

# Building
Build a new virtual environment:

    $ python -m venv .venv
    $ source .venv/bin/activate
    $ pip install maturin
To build the library for development use `maturin develop`:

    $ maturin develop
    $ python
    >>> import ergo_lib_python

# Running tests
Both python unit tests and doc tests are defined
## Run unit tests
     $ python -m unittest tests/*.py
## Run doctests
Running doctests requires installing Sphinx, which is included in `docs/requirements.txt`:
 
    $ pip install -r docs/requirements.txt
    $ cd docs
    $ make doctest
