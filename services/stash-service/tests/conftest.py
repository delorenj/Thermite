"""
Pytest configuration and fixtures for Thermite tests.
"""

import asyncio

import pytest
import pytest_asyncio


@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the event loop for the test session."""
    loop = asyncio.get_event_loop_policy().new_event_loop()
    yield loop
    loop.close()
