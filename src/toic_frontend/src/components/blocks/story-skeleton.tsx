import { Skeleton } from '../ui/skeleton'

export function StorySkeleton() {
  return (
    <div className='flex flex-col space-y-8'>
      <Skeleton className='h-14 w-2xl' />
      <div className='space-y-4'>
        <Skeleton className='h-8 w-xl' />
        <Skeleton className='h-8 w-2xl' />
        <Skeleton className='h-8 w-2xl' />
        <Skeleton className='h-8 w-md' />
      </div>
    </div>
  )
}
