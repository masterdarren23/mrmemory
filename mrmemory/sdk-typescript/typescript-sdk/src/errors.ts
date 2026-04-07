/** Base error for all AMR errors. */
export class AMRError extends Error {
  readonly statusCode?: number

  constructor(message: string, statusCode?: number) {
    super(message)
    this.name = 'AMRError'
    this.statusCode = statusCode
  }
}

/** Invalid or missing API key (401). */
export class AuthenticationError extends AMRError {
  constructor(message = 'Authentication failed') {
    super(message, 401)
    this.name = 'AuthenticationError'
  }
}

/** Too many requests (429). */
export class RateLimitError extends AMRError {
  readonly retryAfter: number

  constructor(message = 'Rate limited', retryAfter = 1) {
    super(message, 429)
    this.name = 'RateLimitError'
    this.retryAfter = retryAfter
  }
}

/** Resource not found (404). */
export class NotFoundError extends AMRError {
  constructor(message = 'Not found') {
    super(message, 404)
    this.name = 'NotFoundError'
  }
}

/** Invalid request parameters (422). */
export class ValidationError extends AMRError {
  constructor(message = 'Validation error') {
    super(message, 422)
    this.name = 'ValidationError'
  }
}

/** AMR server error (5xx). */
export class ServerError extends AMRError {
  constructor(message = 'Server error', statusCode = 500) {
    super(message, statusCode)
    this.name = 'ServerError'
  }
}
