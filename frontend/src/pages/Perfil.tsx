// Perfil (/m/:address): o ponto didático da aula.
// Coluna "on-chain": reputation() + badges_of() lidos DIRETO do contrato (simulação, sem carteira).
// Coluna "off-chain": histórico do Laravel (GET /api/members/{address}), incluindo revogados.
// Cada coluna carrega e falha de forma independente.
import { useEffect, useState } from 'react'
import { useParams } from 'react-router-dom'
import { api, isAbortError, type MemberProfile } from '../api'
import { isValidAccount, readBadges, readReputation, type Badge } from '../contract'
import { formatDateTime, shortAddress } from '../format'

type Load<T> = { kind: 'loading' } | { kind: 'error'; message: string } | { kind: 'ready'; data: T }

const errorMessage = (err: unknown) => (err instanceof Error ? err.message : String(err))

export default function Perfil() {
  const { address = '' } = useParams()
  const valid = isValidAccount(address)

  if (!valid) {
    // Endereço inválido: nenhuma chamada ao contrato nem à API (US-08).
    return (
      <section>
        <h1>Perfil</h1>
        <p className="state error" role="alert">
          Endereço inválido: <span className="mono">{address || '(vazio)'}</span>. Use um endereço
          Stellar de conta, começando com G (56 caracteres).
        </p>
      </section>
    )
  }

  // key={address}: ao trocar de membro, as colunas reiniciam no estado "carregando".
  return (
    <section>
      <h1>Perfil</h1>
      <p className="mono">{address}</p>
      <div className="columns">
        <OnChainColumn key={`on-${address}`} address={address} />
        <OffChainColumn key={`off-${address}`} address={address} />
      </div>
    </section>
  )
}

function OnChainColumn({ address }: { address: string }) {
  const [state, setState] = useState<Load<{ reputation: number; badges: Badge[] }>>({
    kind: 'loading',
  })

  useEffect(() => {
    let cancelled = false
    Promise.all([readReputation(address), readBadges(address)])
      .then(([reputation, badges]) => {
        if (!cancelled) setState({ kind: 'ready', data: { reputation, badges } })
      })
      .catch((err: unknown) => {
        if (!cancelled) setState({ kind: 'error', message: errorMessage(err) })
      })
    return () => {
      cancelled = true
    }
  }, [address])

  return (
    <article className="column onchain" aria-labelledby="onchain-title">
      <span className="tag onchain">on-chain</span>
      <h2 id="onchain-title">Reputação no contrato</h2>
      <p className="source">Lido direto do contrato Selo na testnet (o cartório).</p>

      {state.kind === 'loading' && <p className="state">Lendo o contrato…</p>}

      {state.kind === 'error' && (
        <p className="state error" role="alert">
          Erro ao ler o contrato: {state.message}
        </p>
      )}

      {state.kind === 'ready' && (
        <>
          <p className="big-number">{state.data.reputation} pts</p>
          <h3>Selos ativos ({state.data.badges.length})</h3>
          {state.data.badges.length === 0 ? (
            <p>Nenhum selo ativo.</p>
          ) : (
            <ul className="badge-list">
              {state.data.badges.map((badge) => (
                <li key={`${badge.kind.tag}-${badge.event_id}`}>
                  <strong>{badge.kind.tag}</strong> · evento #{badge.event_id}
                  <span className="detail">
                    emitido em {formatDateTime(new Date(Number(badge.issued_at) * 1000))} por{' '}
                    <span className="mono">{shortAddress(badge.issuer)}</span>
                  </span>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </article>
  )
}

function OffChainColumn({ address }: { address: string }) {
  const [state, setState] = useState<Load<MemberProfile>>({ kind: 'loading' })

  useEffect(() => {
    const controller = new AbortController()
    api
      .member(address, controller.signal)
      .then((data) => setState({ kind: 'ready', data }))
      .catch((err: unknown) => {
        if (isAbortError(err)) return
        setState({ kind: 'error', message: errorMessage(err) })
      })
    return () => controller.abort()
  }, [address])

  return (
    <article className="column offchain" aria-labelledby="offchain-title">
      <span className="tag offchain">off-chain</span>
      <h2 id="offchain-title">Histórico no Laravel</h2>
      <p className="source">Copiado dos eventos do contrato pelo indexador (o arquivo morto).</p>

      {state.kind === 'loading' && <p className="state">Carregando histórico…</p>}

      {state.kind === 'error' && (
        <p className="state error" role="alert">
          Erro ao carregar o histórico: {state.message}
        </p>
      )}

      {state.kind === 'ready' && (
        <>
          {state.data.nickname && <p>Apelido: {state.data.nickname}</p>}
          <p className="big-number">{state.data.points} pts</p>
          <h3>Histórico, inclusive revogados ({state.data.badges.length})</h3>
          {state.data.badges.length === 0 ? (
            <p>Nenhum registro.</p>
          ) : (
            <ul className="badge-list">
              {state.data.badges.map((record) => (
                <li key={`${record.tx_hash}-${record.kind}-${record.issued_at}`}>
                  <span className={record.status === 'revoked' ? 'revoked-item' : undefined}>
                    <strong>{record.kind}</strong> (+{record.weight}) ·{' '}
                    {record.event ? record.event.name : 'evento não cadastrado no Laravel'}
                  </span>{' '}
                  <span className={`status ${record.status}`}>
                    {record.status === 'issued' ? 'ativo' : 'revogado'}
                  </span>
                  <span className="detail">
                    emitido em {formatDateTime(new Date(record.issued_at))}
                    {record.revoked_at &&
                      ` · revogado em ${formatDateTime(new Date(record.revoked_at))}`}
                    {' · ledger '}
                    {record.ledger}
                    {' · tx '}
                    <span className="mono">{shortAddress(record.tx_hash)}</span>
                  </span>
                </li>
              ))}
            </ul>
          )}
        </>
      )}
    </article>
  )
}
