import { AuthClient } from '@dfinity/auth-client'
import { toic_backend } from '@declarations/toic_backend'
import { Principal } from '@dfinity/principal'
import { CanisterEnv } from '../lib/env'

const TTL: bigint = BigInt(1) * BigInt(3_600_000_000_000)

function userExistCheck() {}

class AuthService {
  private static instance: AuthService
  private authClient: AuthClient | null = null

  private constructor() {}

  public static async getInstance() {
    if (!AuthService.instance) {
      AuthService.instance = new AuthService()
      await AuthService.instance.init()
    }
    return AuthService.instance
  }

  private async init() {
    this.authClient = await AuthClient.create()
  }

  /// May throw an error through promise.reject
  public async login() {
    const isAuthed = await this.isAuthenticated()
    if (isAuthed) {
      return
    }

    return new Promise<void>((resolve, reject) => {
      console.log('check', CanisterEnv.identityURL)
      this.authClient?.login({
        identityProvider: CanisterEnv.identityURL,
        maxTimeToLive: TTL,
        onSuccess: () => {
          resolve()
        },
        onError: reject,
        windowOpenerFeatures: ''
      })
    })
  }

  public async logout() {
    await this.authClient?.logout()
  }

  public async isAuthenticated() {
    return this.authClient?.isAuthenticated() ?? false
  }

  public getPrincipal(): Principal | null {
    return this.authClient?.getIdentity().getPrincipal() ?? null
  }

  public getClient(): AuthClient | null {
    return this.authClient
  }
}

/// Get singleton instance of AuthService
async function authService() {
  return AuthService.getInstance()
}

export { authService, type AuthService, TTL }
