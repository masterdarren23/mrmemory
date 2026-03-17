/** A stored memory. */
export interface Memory {
  id: string
  content: string
  tags: string[]
  namespace: string
  agentId: string
  /** Similarity score — present only on recall results. */
  score?: number
  ttl?: string | null
  expiresAt?: Date | null
  createdAt: Date
  updatedAt: Date
}

/** A real-time memory event from the WebSocket stream. */
export interface MemoryEvent {
  type: 'memory.created' | 'memory.shared' | 'memory.expired'
  memoryId: string
  memory?: Memory
  timestamp: Date
}

/** Options for remember(). */
export interface RememberOptions {
  tags?: string[]
  namespace?: string
  agentId?: string
  /** TTL as human string ('30d', '12h', '5m') or milliseconds. */
  ttl?: string | number
}

/** Options for recall(). */
export interface RecallOptions {
  namespace?: string
  agentId?: string
  tags?: string[]
  /** Max results (default: 5). */
  limit?: number
  /** Min similarity score (default: 0.7). */
  threshold?: number
}

/** Options for share(). */
export interface ShareOptions {
  targetAgent: string
  permissions?: 'read' | 'readwrite'
}

/** Options for forget_all(). */
export interface ForgetAllOptions {
  namespace?: string
}

/** Options for memories(). */
export interface ListOptions {
  namespace?: string
  agentId?: string
  tags?: string[]
  limit?: number
  offset?: number
}

/** Client configuration. */
export interface AMRConfig {
  apiKey: string
  baseUrl?: string
  agentId?: string
  namespace?: string
  /** Request timeout in ms (default: 10000). */
  timeout?: number
  /** Max retries on transient failures (default: 3). */
  maxRetries?: number
}
