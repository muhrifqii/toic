const s = 'localhost:4943/'

const network = import.meta.env.DFX_NETWORK as string

const CanisterEnv = {
  identityURL:
    network === 'ic'
      ? 'https://identity.ic0.app'
      : `http://${import.meta.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:4943`
}

export { CanisterEnv }
