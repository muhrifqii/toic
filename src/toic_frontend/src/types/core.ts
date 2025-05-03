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
