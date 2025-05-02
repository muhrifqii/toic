import { cn } from '@/lib/utils'
import { FC, HTMLAttributes } from 'react'

function Container({ className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return <div className={cn('mx-auto max-w-7xl px-4 sm:px-6 lg:px-8', className)} {...props} />
}

export { Container }
