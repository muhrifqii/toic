import { CandidOption, CandidResult, ApiResult } from '@/types/candid'
import { CategoryName } from '@/types/core'
import { Category } from '@declarations/toic_backend/toic_backend.did'

export function optionOf<T>(value?: T | null): CandidOption<T> {
  if (value != null) {
    return [value]
  }
  return []
}

export function mapToCategory(category: CategoryName): Category {
  return { [category]: null } as Category
}

export function mapFromCategory(category: Category): CategoryName {
  return Object.keys(category)[0] as CategoryName
}

export function unwrapResult<T, E>(result: CandidResult<T, E>): [T, null] | [null, E] {
  if ('Ok' in result) {
    return [result.Ok, null]
  } else {
    return [null, result.Err]
  }
}

export function unwrapOption<T>(option?: CandidOption<T>): T | null {
  if (!option) return null

  return option.length === 0 ? null : option[0]
}
