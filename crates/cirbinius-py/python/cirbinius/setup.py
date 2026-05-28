import os
import shutil
from setuptools import setup

PKG_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT = os.path.abspath(os.path.join(PKG_DIR, "..", "..", ".."))

setup(
    name="cirbinius",
    version="0.1.0",
    packages=["cirbinius"],
    package_dir={"cirbinius": PKG_DIR},
    package_data={"cirbinius": ["libcirbinius_py.so"]},
    python_requires=">=3.10",
)
