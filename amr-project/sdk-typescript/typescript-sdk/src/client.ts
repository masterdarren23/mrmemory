import type {
  AMRConfig,
  AutoRememberOptions,
  ChatMessage,
  CompressOptions,
  ForgetAllOptions,
  ListOptions,
  Memory,
  RecallOptions,
  RememberOptions,
  ShareOptions,
} from './types.js'
import { Transport } from './transport.js'
import { parseMemory, parseTtl } from './utils.js'

const DEFAULT_BASE_URL = 'https://api.amr.dev/v1'
const DEFAULT_TIMEOUT = 10_000
const DEFAULT_MAX_RETRIES = 3

/**
 * AMR client — persistent long-term memory for AI agents.
 *
 * @example
 * ```ts
 * import { AMR } from '@amr/client'
 *
 * const amr = new AMR('amr_sk_...')
 * await amr.remember('User prefers dark mode', { tags: ['preferences'] })
 * const memories = await amr.recall('What does the user prefer?')
 * ```
 */
export class AMR {
  private readonly transport: Transport
  private readonly defaultAgentId?: string
  private readonly defaultNamespace?: string

  constructor(config: string | AMRConfig) {
    const cfg = typeof config === 'string' ? { apiKey: config } : config

    if (!cfg.apiKey) {
      // Try environment (Node/Deno/Bun)
      const envKey = typeof process !== 'undefined'
        ? process.env.AMR_API_KEY
        : undefined
      if (!envKey) {
        throw new Error('No API key provided. Pass apiKey or set AMR_API_KEY environment variable.')
      }
      cfg.apiKey = envKey
    }

    this.defaultAgentId = cfg.agentId
    this.defaultNamespace = cfg.namespace

    this.transport = new Transport({
      baseUrl: (cfg.baseUrl ?? DEFAULT_BASE_URL).replace(/\/+$/, ''),
      apiKey: cfg.apiKey,
      timeout: cfg.timeout ?? DEFAULT_TIMEOUT,
      maxRetries: cfg.maxRetries ?? DEFAULT_MAX_RETRIES,
    })
  }

  /** Store a memory. */
  async remember(content: string, options?: RememberOptions): Promise<Memory> {
    const body: Record<string, unknown> = { content }
    if (options?.tags) body.tags = options.tags
    body.namespace = options?.namespace ?? this.defaultNamespace ?? undefined
    body.agent_id = options?.agentId ?? this.defaultAgentId ?? undefined
    if (options?.ttl != null) body.ttl = parseTtl(options.ttl)

    // Strip undefined values
    for (const k of Object.keys(body)) {
      if (body[k] === undefined) delete body[k]
    }

    const data = await this.transport.request('POST', '/remember', { json: body })
    return parseMemory(data as Record<string, unknown>)
  }

  /** Retrieve relevant memories via semantic search. */
  async recall(query: string, options?: RecallOptions): Promise<Memory[]> {
    const body: Record<string, unknown> = {
      query,
      limit: options?.limit ?? 5,
      threshold: options?.threshold ?? 0.7,
    }
    if (options?.tags) body.tags = options.tags
    const ns = options?.namespace ?? this.defaultNamespace
    if (ns) body.namespace = ns
    const aid = options?.agentId ?? this.defaultAgentId
    if (aid) body.agent_id = aid

    const data = await this.transport.request('POST', '/recall', { json: body }) as Record<string, unknown>
    const list = (data.memories ?? data) as Record<string, unknown>[]
    return list.map(parseMemory)
  }

  /** Share memories with another agent. */
  async share(memoryIds: string | string[], options: ShareOptions): Promise<void> {
    const ids = typeof memoryIds === 'string' ? [memoryIds] : memoryIds
    await this.transport.request('POST', '/share', {
      json: {
        memory_ids: ids,
        target_agent_id: options.targetAgent,
        permissions: options.permissions ?? 'read',
      },
    })
  }

  /** Delete one or more memories. */
  async forget(memoryIds: string | string[]): Promise<void> {
    const ids = typeof memoryIds === 'string' ? [memoryIds] : memoryIds
    await this.transport.request('DELETE', '/forget', {
      json: { memory_ids: ids },
    })
  }

  /** Delete all memories, optionally scoped to a namespace. */
  async forgetAll(options?: ForgetAllOptions): Promise<void> {
    const body: Record<string, unknown> = { forget_all: true }
    if (options?.namespace) body.namespace = options.namespace
    await this.transport.request('DELETE', '/forget', { json: body })
  }

  /** List memories with optional filters. */
  async memories(options?: ListOptions): Promise<Memory[]> {
    const params: Record<string, string | number> = {
      limit: options?.limit ?? 20,
      offset: options?.offset ?? 0,
    }
    const ns = options?.namespace ?? this.defaultNamespace
    if (ns) params.namespace = ns
    const aid = options?.agentId ?? this.defaultAgentId
    if (aid) params.agent_id = aid
    if (options?.tags) params.tags = options.tags.join(',')

    const data = await this.transport.request('GET', '/memories', { params }) as Record<string, unknown>
    const list = (data.memories ?? data) as Record<string, unknown>[]
    return list.map(parseMemory)
  }

  /**
   * Extract and store memories from conversation messages using LLM.
   * Supports async (fire-and-forget) and sync modes. BYOK supported.
   */
  async autoRemember(messages: ChatMessage[], options?: AutoRememberOptions): Promise<Record<string, unknown>> {
    const body: Record<string, unknown> = { messages }
    const ns = options?.namespace ?? this.defaultNamespace
    if (ns) body.namespace = ns
    const aid = options?.agentId ?? this.defaultAgentId
    if (aid) body.agent_id = aid
    if (options?.llmApiKey) body.llm_api_key = options.llmApiKey
    if (options?.llmModel) body.llm_model = options.llmModel
    if (options?.sync) body.sync = true

    return await this.transport.request('POST', '/memories/auto', { json: body }) as Record<string, unknown>
  }

  /**
   * Compress related memories in a namespace using LLM summarization.
   * Groups semantically similar memories and merges each group into one.
   */
  async compress(options?: CompressOptions): Promise<Record<string, unknown>> {
    const body: Record<string, unknown> = {}
    const ns = options?.namespace ?? this.defaultNamespace
    if (ns) body.namespace = ns
    const aid = options?.agentId ?? this.defaultAgentId
    if (aid) body.agent_id = aid
    if (options?.llmApiKey) body.llm_api_key = options.llmApiKey
    if (options?.llmModel) body.llm_model = options.llmModel
    if (options?.threshold != null) body.threshold = options.threshold
    if (options?.similarityThreshold != null) body.similarity_threshold = options.similarityThreshold
    if (options?.sync) body.sync = true
    if (options?.dryRun) body.dry_run = true

    return await this.transport.request('POST', '/memories/compress', { json: body }) as Record<string, unknown>
  }
}
