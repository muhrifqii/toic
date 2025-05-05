import { AuthClient } from '@dfinity/auth-client'
import { Principal } from '@dfinity/principal'
import { CanisterEnv } from '@/lib/env'
import { unwrapResult } from '@/lib/mapper'
import { OnboardingArgs, User } from '@declarations/toic_backend/toic_backend.did'
import { canisterId, createActor, toic_backend } from '@declarations/toic_backend'

const TTL: bigint = BigInt(1) * BigInt(3_600_000_000_000)

class AuthService {
  private static instance: AuthService
  private authClient: AuthClient | null = null
  private user: User | null = null
  private actor: typeof toic_backend | null = null

  private constructor() {}

  public static async getInstance() {
    if (!AuthService.instance) {
      AuthService.instance = new AuthService()
      await AuthService.instance.invalidateActor()
    }
    return AuthService.instance
  }

  private async invalidateActor() {
    this.authClient = await AuthClient.create()
    this.resetActor()
  }

  private resetActor() {
    this.actor = createActor(canisterId, { agentOptions: { identity: this.authClient?.getIdentity() } })
  }

  /// May throw an error through promise.reject
  public async login() {
    const isAuthed = await this.isAuthenticated()
    if (isAuthed) {
      return
    }

    const onSuccess = async (resolve: () => void, reject: (reason: string) => void) => {
      await this.invalidateActor()
      const err = await this.backendLogin()
      if (err) {
        return reject(err.message)
      }
      resolve()
    }

    return new Promise<void>((resolve, reject) => {
      this.authClient?.login({
        identityProvider: CanisterEnv.identityURL,
        maxTimeToLive: TTL,
        onSuccess: () => onSuccess(resolve, reject),
        onError: reject
      })
    })
  }

  public async logout() {
    await this.authClient?.logout()
    this.user = null
    await this.invalidateActor()
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

  public getActor(): typeof toic_backend {
    return this.actor!
  }

  public getUser(): User | null {
    return this.user
  }

  public async onboard(args: OnboardingArgs) {
    const result = await this.getActor().complete_onboarding(args)
    const [withReferral, err] = unwrapResult(result)
    if (!!err) {
      throw err.message
    }
    return withReferral
  }

  public async backendLogin() {
    const result = await this.getActor().login()
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
