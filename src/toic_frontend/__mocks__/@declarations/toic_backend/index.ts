import { User } from '@declarations/toic_backend/toic_backend.did'

export const canisterId = 'FOOBAR'

export const toic_backend = {
  login: async () => {
    console.log('Mocked @declarations/toic_backend is being used')
    return { Ok: {} as User }
  }
}

export const createActor = (can: string, opt: { agentOptions: { identity: unknown } }) => {
  return toic_backend
}
