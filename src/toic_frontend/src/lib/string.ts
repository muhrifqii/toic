import { Buffer } from 'buffer'
import { format as fns } from 'date-fns'

export function encodeId(value: BigInt) {
  const str = value.toString()
  return Buffer.from(str, 'binary').toString('base64')
}

export function decodeId(value: string) {
  const result = Buffer.from(value, 'base64').toString('binary')
  return BigInt(result)
}

export function tokenDisplay(token?: bigint | string | number | null) {
  const numericValue = Number(token ?? 0)
  return numericValue.toLocaleString('en-US')
}

export function formatDate(nanos: bigint, fmt: string = 'dd MMM yyyy') {
  return fns(Number(nanos / 1_000_000n), fmt)
}
