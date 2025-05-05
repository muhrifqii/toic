import { AuthClient } from '@dfinity/auth-client'
import { toic_backend } from '@declarations/toic_backend'
import { Principal } from '@dfinity/principal'
import { CanisterEnv } from '@/lib/env'
import { unwrapResult } from '@/lib/mapper'
import { OnboardingArgs, User } from '@declarations/toic_backend/toic_backend.did'

const TTL: bigint = BigInt(1) * BigInt(3_600_000_000_000)

class AuthService {
  private static instance: AuthService
  private authClient: AuthClient | null = null
  private user: User | null = null

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
      this.authClient?.login({
        identityProvider: CanisterEnv.identityURL,
        maxTimeToLive: TTL,
        onSuccess: async () => {
          const err = await this.backendLogin()
          if (err) {
            return reject(err.message)
          }
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

  public getUser(): User | null {
    return this.user
  }

  public async onboard(args: OnboardingArgs) {
    const result = await toic_backend.complete_onboarding(args)
    const [withReferral, err] = unwrapResult(result)
    if (!!err) {
      throw err.message
    }
    return withReferral
  }

  public async backendLogin() {
    const result = await toic_backend.login()
    const [user, err] = unwrapResult(result)
    if (!!err) {
      return err
    }
    this.user = user
  }
}

/// Get singleton instance of AuthService
async function authService() {
  return AuthService.getInstance()
}

export { authService, type AuthService, TTL }
