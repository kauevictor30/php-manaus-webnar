// Ranking (/): top 20 por pontos de badges ativos, vindo do Laravel (GET /api/ranking).
// Não exige carteira.
import { useEffect, useState } from 'react'
import { Link } from 'react-router-dom'
import { api, isAbortError, type RankingRow } from '../api'
import { shortAddress } from '../format'

type State =
  | { kind: 'loading' }
  | { kind: 'error'; message: string }
  | { kind: 'ready'; rows: RankingRow[] }

export default function Ranking() {
  const [state, setState] = useState<State>({ kind: 'loading' })

  useEffect(() => {
    const controller = new AbortController()
    api
      .ranking(controller.signal)
      .then((rows) => setState({ kind: 'ready', rows }))
      .catch((err: unknown) => {
        if (isAbortError(err)) return
        setState({ kind: 'error', message: err instanceof Error ? err.message : String(err) })
      })
    return () => controller.abort()
  }, [])

  return (
    <section>
      <h1>Ranking</h1>
      <p className="source">Top 20 por pontos de selos ativos · fonte: Laravel (off-chain)</p>

      {state.kind === 'loading' && <p className="state">Carregando ranking…</p>}

      {state.kind === 'error' && (
        <p className="state error" role="alert">
          Erro ao carregar o ranking: {state.message}
        </p>
      )}

      {state.kind === 'ready' && state.rows.length === 0 && (
        <p className="state">Nenhum selo ainda.</p>
      )}

      {state.kind === 'ready' && state.rows.length > 0 && (
        <ol className="ranking">
          {state.rows.map((row) => (
            <li key={row.address}>
              <Link to={`/m/${row.address}`}>
                <span className="position">{row.position}º</span>
                <span className="who">
                  <strong>{row.nickname ?? shortAddress(row.address)}</strong>
                  <span className="mono detail">{row.address}</span>
                </span>
                <span className="points">
                  {row.points} pts
                  <small>
                    {row.badges} {row.badges === 1 ? 'selo' : 'selos'}
                  </small>
                </span>
              </Link>
            </li>
          ))}
        </ol>
      )}
    </section>
  )
}
