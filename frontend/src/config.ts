// Toda configuração vem das variáveis de ambiente (.env.local). Nada hardcoded.
type EnvKey =
  | 'VITE_SELO_CONTRACT_ID'
  | 'VITE_RPC_URL'
  | 'VITE_NETWORK_PASSPHRASE'
  | 'VITE_API_URL'

function requireEnv(key: EnvKey): string {
  const value = import.meta.env[key]
  if (!value) {
    throw new Error(`Variável de ambiente ${key} não definida (veja frontend/.env.example)`)
  }
  return value
}

// Getters: a falta de uma env só quebra a parte que depende dela (ex.: Ranking funciona sem contract ID).
export const config = {
  get contractId() {
    return requireEnv('VITE_SELO_CONTRACT_ID')
  },
  get rpcUrl() {
    return requireEnv('VITE_RPC_URL')
  },
  get networkPassphrase() {
    return requireEnv('VITE_NETWORK_PASSPHRASE')
  },
  get apiUrl() {
    return requireEnv('VITE_API_URL').replace(/\/+$/, '')
  },
}

export const explorerTxUrl = (hash: string) =>
  `https://stellar.expert/explorer/testnet/tx/${hash}`
