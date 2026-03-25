"""
Public Python access point for the P4 Rust core bindings.
"""

from bindings.python.p4_core import P4Core, P4CoreError

__all__ = ["P4Core", "P4CoreError"]


