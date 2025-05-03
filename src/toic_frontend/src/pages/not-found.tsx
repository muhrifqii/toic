import { Button } from '@/components/ui/button'
import { Link } from 'react-router'

export default function NotFoundPage() {
  return (
    <main>
      <div className='flex flex-col items-center justify-center h-screen text-center'>
        <h1 className='text-6xl font-bold text-gray-800'>404</h1>
        <p className='mt-4 text-lg text-gray-600'>Page not found</p>
        <Link to='/'>
          <Button variant='link' className='mt-6 px-4'>
            Back to basic
          </Button>
        </Link>
      </div>
    </main>
  )
}
