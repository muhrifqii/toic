import { StateCreator, StoreMutatorIdentifier } from 'zustand/vanilla'
import { devtools } from 'zustand/middleware'

export const withMiddleware = <
  T,
  Mps extends [StoreMutatorIdentifier, unknown][] = [],
  Mcs extends [StoreMutatorIdentifier, unknown][] = [],
  U = T
>(
  f: StateCreator<T, [...Mps, ['zustand/devtools', never]], Mcs, U>
) => devtools(f)
