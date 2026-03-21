"""
Interface Module for Unified Mind.

Provides user interfaces for interacting with the Unified Mind AI agent.
"""

from .cli import CLIInterface

try:
    from .web import WebInterface
except ImportError:  # Optional interface module
    WebInterface = None

__all__ = ["CLIInterface", "WebInterface"]
