"""AMR exception hierarchy."""

from __future__ import annotations


class AMRError(Exception):
    """Base exception for all AMR errors."""

    def __init__(self, message: str, status_code: int | None = None) -> None:
        super().__init__(message)
        self.status_code = status_code


class AuthenticationError(AMRError):
    """Invalid or missing API key (401)."""


class RateLimitError(AMRError):
    """Too many requests (429)."""

    def __init__(self, message: str, retry_after: float = 1.0) -> None:
        super().__init__(message, status_code=429)
        self.retry_after = retry_after


class NotFoundError(AMRError):
    """Resource not found (404)."""

    def __init__(self, message: str = "Not found") -> None:
        super().__init__(message, status_code=404)


class ValidationError(AMRError):
    """Invalid request parameters (422)."""

    def __init__(self, message: str = "Validation error") -> None:
        super().__init__(message, status_code=422)


class ServerError(AMRError):
    """AMR server error (5xx)."""

    def __init__(self, message: str = "Server error", status_code: int = 500) -> None:
        super().__init__(message, status_code=status_code)
