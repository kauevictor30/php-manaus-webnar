// Cliente da API Laravel (o "arquivo morto"). Formatos conforme design.md §3.3.
import { config } from './config'

export type BadgeKindName = 'Participante' | 'Mentor' | 'Organizador' | 'Palestrante'

export interface ApiEvent {
  id: number
  name: string
  date: string
  location: string
  description: string | null
}

export interface MemberBadgeRecord {
  kind: BadgeKindName
  weight: number
  event: { id: number; name: string } | null
  issuer: string
  status: 'issued' | 'revoked'
  tx_hash: string
  ledger: number
  issued_at: string
  revoked_at: string | null
}

export interface MemberProfile {
  address: string
  nickname: string | null
  avatar_url: string | null
  points: number
  badges: MemberBadgeRecord[]
}

export interface RankingRow {
  position: number
  address: string
  nickname: string | null
  points: number
  badges: number
}

export class ApiError extends Error {
  readonly status: number | null

  constructor(message: string, status: number | null) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

export function isAbortError(err: unknown): boolean {
  return err instanceof DOMException && err.name === 'AbortError'
}

async function getJson<T>(path: string, signal?: AbortSignal): Promise<T> {
  let response: Response
  try {
    response = await fetch(`${config.apiUrl}${path}`, {
      headers: { Accept: 'application/json' },
      signal,
    })
  } catch (err) {
    if (isAbortError(err)) throw err
    throw new ApiError('Não foi possível falar com a API (o Laravel está rodando?)', null)
  }
  if (!response.ok) {
    throw new ApiError(`A API respondeu com erro ${response.status}`, response.status)
  }
  return (await response.json()) as T
}

export const api = {
  ranking: (signal?: AbortSignal) => getJson<RankingRow[]>('/ranking', signal),
  member: (address: string, signal?: AbortSignal) =>
    getJson<MemberProfile>(`/members/${encodeURIComponent(address)}`, signal),
  events: (signal?: AbortSignal) => getJson<ApiEvent[]>('/events', signal),
}
