import { Loader2 } from 'lucide-react'
import React from 'react'

export default function LoadingPage() {
  return (
    <div className='flex flex-col items-center justify-center h-screen bg-background'>
      <Loader2 className='w-12 h-12 animate-spin' />
      <p className='mt-4 text-lg text-shadow-foreground'>Loading...</p>
    </div>
  )
}
