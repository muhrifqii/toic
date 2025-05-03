export const categoryNames = [
  'SciFi',
  'Adventure',
  'NonFiction',
  'Romance',
  'Fantasy',
  'Crime',
  'Biography',
  'Thriller',
  'Comedy',
  'Horror'
] as const

export type CategoryName = (typeof categoryNames)[number]

export type OnboardingArgsBuilder = {
  name?: string
  bio?: string
  categories: CategoryName[]
}
