import { Buffer } from 'buffer'

export function encodeId(value: BigInt) {
  const str = value.toString()
  return Buffer.from(str, 'binary').toString('base64')
}

export function decodeId(value: string) {
  const result = Buffer.from(value, 'base64').toString('binary')
  return BigInt(result)
}
