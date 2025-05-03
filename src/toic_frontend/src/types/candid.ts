import { ErrorResponse } from '@declarations/toic_backend/toic_backend.did'

type CandidOption<T> = [] | [T]
type CandidResult<T, E> = { Ok: T } | { Err: E }
type ApiResult<T> = CandidResult<T, ErrorResponse>

export type { CandidOption, CandidResult, ApiResult }
