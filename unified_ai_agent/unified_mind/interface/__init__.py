"""
Interface Module for Unified Mind.

Provides user interfaces for interacting with the Unified Mind AI agent.
"""

from .cli import CLIInterface
from .web import WebInterface

__all__ = ["CLIInterface", "WebInterface"]
