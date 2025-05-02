export interface MockLoginOptions {
  identityProvider?: string
  maxTimeToLive?: bigint
  onSuccess?: () => void | Promise<void>
  onError?: (error?: string) => void
}

export class MockIdentity {
  getPrincipal() {
    return { toText: () => 'mock-principal-id' }
  }
}

export class AuthClient {
  static async create() {
    return new AuthClient()
  }

  async isAuthenticated() {
    return true
  }

  async login({ onSuccess }: MockLoginOptions) {
    onSuccess?.()
  }

  async logout() {}

  getIdentity() {
    return new MockIdentity()
  }
}
