"""MrMemory — alias for the AMR package. Both `from amr import AMR` and `from mrmemory import AMR` work."""

from amr.client import AMR
from amr.async_client import AsyncAMR
from amr.types import Memory, MemoryEvent

__all__ = ["AMR", "AsyncAMR", "Memory", "MemoryEvent"]


def _lazy_langchain():
    """Import langchain integration (requires mrmemory[langchain])."""
    from mrmemory.langchain import MrMemoryCheckpointer, MrMemoryStore
    return MrMemoryCheckpointer, MrMemoryStore
