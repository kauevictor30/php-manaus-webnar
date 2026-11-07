// Client do contrato Selo a partir dos bindings gerados (`stellar contract bindings typescript`).
// O contract ID vem SEMPRE da env: os bindings fixam um ID em networks.testnet, que não usamos.
import { Client, StrKey } from 'selo'
import type { Badge } from 'selo'
import { config } from './config'

export type { Badge }

export function seloClient(publicKey?: string): Client {
  return new Client({
    contractId: config.contractId,
    networkPassphrase: config.networkPassphrase,
    rpcUrl: config.rpcUrl,
    publicKey,
  })
}

/** Endereço de conta G... válido (o "passaporte"). */
export function isValidAccount(address: string): boolean {
  return StrKey.isValidEd25519PublicKey(address)
}

/** Leitura por simulação, sem carteira: o valor vem em `.result` da AssembledTransaction. */
export async function readReputation(member: string): Promise<number> {
  const tx = await seloClient().reputation({ member })
  return tx.result
}

export async function readBadges(member: string): Promise<Badge[]> {
  const tx = await seloClient().badges_of({ member })
  return tx.result
}
