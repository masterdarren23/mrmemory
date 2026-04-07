import {
  AMRError,
  AuthenticationError,
  NotFoundError,
  RateLimitError,
  ServerError,
  ValidationError,
} from './errors.js'

const RETRYABLE = new Set([429, 500, 502, 503, 504])

interface TransportConfig {
  baseUrl: string
  apiKey: string
  timeout: number
  maxRetries: number
}

function backoff(attempt: number): number {
  const base = Math.min(2 ** attempt, 30)
  return base * (0.5 + Math.random() * 0.5)
}

async function raiseForStatus(response: Response): Promise<void> {
  if (response.ok) return

  let message: string
  try {
    const body = await response.json() as Record<string, unknown>
    message = (body.error ?? body.message ?? response.statusText) as string
  } catch {
    message = response.statusText || `HTTP ${response.status}`
  }

  const code = response.status
  if (code === 401) throw new AuthenticationError(message)
  if (code === 404) throw new NotFoundError(message)
  if (code === 422) throw new ValidationError(message)
  if (code === 429) {
    const retryAfter = parseFloat(response.headers.get('retry-after') ?? '1')
    throw new RateLimitError(message, retryAfter)
  }
  if (code >= 500) throw new ServerError(message, code)
  throw new AMRError(message, code)
}

export class Transport {
  private readonly config: TransportConfig

  constructor(config: TransportConfig) {
    this.config = config
  }

  async request(method: string, path: string, options?: {
    json?: unknown
    params?: Record<string, string | number>
  }): Promise<unknown> {
    const { baseUrl, apiKey, timeout, maxRetries } = this.config

    let url = `${baseUrl}${path}`
    if (options?.params) {
      const qs = new URLSearchParams()
      for (const [k, v] of Object.entries(options.params)) {
        qs.set(k, String(v))
      }
      url += `?${qs.toString()}`
    }

    const headers: Record<string, string> = {
      'Authorization': `Bearer ${apiKey}`,
      'Content-Type': 'application/json',
      'User-Agent': 'amr-typescript/0.1.0',
    }

    let lastError: Error | null = null

    for (let attempt = 0; attempt <= maxRetries; attempt++) {
      try {
        const controller = new AbortController()
        const timer = setTimeout(() => controller.abort(), timeout)

        const response = await fetch(url, {
          method,
          headers,
          body: options?.json ? JSON.stringify(options.json) : undefined,
          signal: controller.signal,
        })

        clearTimeout(timer)

        if (RETRYABLE.has(response.status) && attempt < maxRetries) {
          const wait = parseFloat(response.headers.get('retry-after') ?? '0') || backoff(attempt)
          await sleep(wait * 1000)
          continue
        }

        await raiseForStatus(response)

        if (response.status === 204) return null
        return await response.json()
      } catch (e) {
        if (e instanceof AMRError) {
          // Non-retryable AMR errors — throw immediately
          throw e
        }
        lastError = e as Error
        if (attempt < maxRetries) {
          await sleep(backoff(attempt) * 1000)
          continue
        }
      }
    }

    throw lastError ?? new AMRError('Request failed after retries')
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}
