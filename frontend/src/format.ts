// Formatação compartilhada entre as telas.

export function shortAddress(address: string): string {
  return address.length > 12 ? `${address.slice(0, 6)}…${address.slice(-6)}` : address
}

export function formatDateTime(date: Date): string {
  return date.toLocaleString('pt-BR', { dateStyle: 'short', timeStyle: 'short' })
}
