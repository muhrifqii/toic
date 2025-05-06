import { ErrorResponse } from '@declarations/toic_backend/toic_backend.did'

type CandidOption<T> = [] | [T]
type CandidResult<T, E> = { Ok: T } | { Err: E }
type ApiResult<T> = CandidResult<T, ErrorResponse>

type CursorType<T> = [] | [T]
type DoubleCursorType<T, U> = [] | [[T, U]]

type IdCursorTuple<T> = [CursorType<bigint>, Array<T>]
type IdTimestampCursorTuple<T> = [DoubleCursorType<bigint, bigint>, Array<T>]

export type {
  CandidOption,
  CandidResult,
  ApiResult,
  CursorType,
  DoubleCursorType,
  IdCursorTuple,
  IdTimestampCursorTuple
}
