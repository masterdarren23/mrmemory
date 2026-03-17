import type { Memory } from './types.js'

/** Parse a TTL string ('30d', '12h', '5m') or ms number into seconds. */
export function parseTtl(ttl: string | number): number {
  if (typeof ttl === 'number') return Math.floor(ttl / 1000)

  const match = ttl.match(/^(\d+)(s|m|h|d)$/)
  if (!match) throw new Error(`Invalid TTL format: "${ttl}". Use '30d', '12h', '5m', '60s', or ms number.`)

  const value = parseInt(match[1], 10)
  const unit = match[2]

  switch (unit) {
    case 's': return value
    case 'm': return value * 60
    case 'h': return value * 3600
    case 'd': return value * 86400
    default: throw new Error(`Unknown TTL unit: ${unit}`)
  }
}

/** Parse an API response dict into a Memory object. */
export function parseMemory(data: Record<string, unknown>): Memory {
  return {
    id: data.id as string,
    content: data.content as string,
    tags: (data.tags as string[]) ?? [],
    namespace: (data.namespace as string) ?? 'default',
    agentId: (data.agent_id as string) ?? '',
    score: data.score as number | undefined,
    ttl: data.ttl != null ? String(data.ttl) : null,
    expiresAt: data.expires_at ? new Date(data.expires_at as string) : null,
    createdAt: new Date((data.created_at as string) ?? Date.now()),
    updatedAt: new Date((data.updated_at as string) ?? Date.now()),
  }
}
