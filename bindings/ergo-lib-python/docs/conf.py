# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information

import ergo_lib_python
project = 'ergo-lib-python'
copyright = '2025, SethDusek (Kamal Ahmad)'
author = 'SethDusek (Kamal Ahmad)'
release = ergo_lib_python.__version__

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration

extensions = ['autoapi.extension', 'sphinx.ext.doctest']
doctest_global_setup = "from ergo_lib_python import *; from ergo_lib_python.chain import *; from ergo_lib_python.wallet import *; from ergo_lib_python.transaction import *"
autoapi_dirs = ['../ergo_lib_python']
autoapi_file_patterns = ['*.pyi']
autoapi_member_order = 'groupwise'
autoapi_python_class_content = 'both'
autoapi_add_toctree_entry = False
autodoc_typehints = 'both'
templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store']



# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output

html_theme = 'sphinx_rtd_theme'
html_static_path = ['_static']
html_css_files = ["propertyfix.css"]
