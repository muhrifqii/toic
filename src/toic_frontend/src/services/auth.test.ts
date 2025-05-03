import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { authService, AuthService } from '../services/auth'

vi.mock('@dfinity/auth-client')
vi.mock('@declarations/toic_backend', () => {
  return import('../../__mocks__/@declarations/toic_backend')
})

describe('AuthService', () => {
  let auth: AuthService

  beforeEach(async () => {
    auth = await authService()
  })

  it('should report authenticated', async () => {
    expect(await auth.isAuthenticated()).toBe(true)
  })

  it('should return mock principal', () => {
    const principal = auth.getPrincipal()
    expect(principal?.toText()).toBe('mock-principal-id')
  })

  it('should expose the AuthClient instance', () => {
    expect(auth.getClient()).not.toBeNull()
  })

  it('should skip login if already authenticated', async () => {
    const authedSpy = vi.spyOn(auth.getClient()!, 'isAuthenticated')
    await auth.login()
    expect(authedSpy).toHaveBeenCalled()
  })

  it('should support login when unauthenticated', async () => {
    auth.isAuthenticated = vi.fn(async () => false)
    const loginSpy = vi.spyOn(auth.getClient()!, 'login')
    await auth.login()
    expect(loginSpy).toHaveBeenCalled()
  })

  it('should support logout', async () => {
    const logoutSpy = vi.spyOn(auth.getClient()!, 'logout')
    await auth.logout()
    expect(logoutSpy).toHaveBeenCalled()
  })
})
