import { AuthClient } from '@dfinity/auth-client'
import { toic_backend } from '@declarations/toic_backend'
import { Principal } from '@dfinity/principal'

const TTL: bigint = BigInt(1) * BigInt(3_600_000_000_000)

function userExistCheck() {}

class AuthService {
  private static instance: AuthService
  private authClient: AuthClient | null = null

  private constructor() {}

  public static getInstance() {
    if (!AuthService.instance) {
      AuthService.instance = new AuthService()
      AuthService.instance.init()
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
      this.authClient?.login({
        identityProvider: import.meta.env.VITE_II_CANISTER_URL,
        maxTimeToLive: TTL,
        onSuccess: () => {
          resolve()
        },
        onError: reject
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
