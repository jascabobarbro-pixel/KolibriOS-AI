"""Pytest helpers local to the Unified Mind test suite."""

from __future__ import annotations

import asyncio
import inspect


def pytest_configure(config) -> None:
    """Register async markers used by the local suite."""
    config.addinivalue_line(
        "markers",
        "asyncio: run the marked test function inside an asyncio event loop",
    )


def pytest_pyfunc_call(pyfuncitem):
    """Run async test functions without depending on pytest-asyncio."""
    test_function = pyfuncitem.obj
    if not inspect.iscoroutinefunction(test_function):
        return None

    kwargs = {
        arg: pyfuncitem.funcargs[arg]
        for arg in pyfuncitem._fixtureinfo.argnames
    }
    asyncio.run(test_function(**kwargs))
    return True
