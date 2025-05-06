import { describe, it, expect } from 'vitest'
import { encodeId, decodeId, tokenDisplay } from './string'

describe('string.ts', () => {
  it('should encode a BigInt to a base64 string', () => {
    const input = BigInt(1)
    const encoded = encodeId(input)
    expect(encoded).toBe(Buffer.from(input.toString(), 'binary').toString('base64'))
  })

  it('should decode a base64 string to a BigInt', () => {
    const input = BigInt(3)
    const encoded = encodeId(input)
    const decoded = decodeId(encoded)
    expect(encoded).toBe('Mw==')
    expect(decoded).toBe(input)
  })

  it('should handle encoding and decoding of large BigInt values', () => {
    const input = BigInt('987654321987654321987654321')
    const encoded = encodeId(input)
    const decoded = decodeId(encoded)
    expect(decoded).toBe(input)
  })

  it('should throw an error when decoding an invalid base64 string', () => {
    const invalidBase64 = 'invalid_base64'
    expect(() => decodeId(invalidBase64)).toThrow()
  })

  it('should format a BigInt token with commas', () => {
    const token = BigInt('1234567890')
    const formatted = tokenDisplay(token)
    expect(formatted).toBe('1,234,567,890')
  })

  it('should format a number token with commas', () => {
    const token = 1234567
    const formatted = tokenDisplay(token)
    expect(formatted).toBe('1,234,567')
  })

  it('should format a string token with commas', () => {
    const token = '9876543210'
    const formatted = tokenDisplay(token)
    expect(formatted).toBe('9,876,543,210')
  })

  it('should return "0" for null or undefined token', () => {
    expect(tokenDisplay(null)).toBe('0')
    expect(tokenDisplay(undefined)).toBe('0')
  })
})
