import { Button } from '@/components/ui/button'
import { authService } from '@/services/auth'
import { useAuthStore } from '@/store/auth'
import { toast } from 'sonner'
import LoadingPage from './loading'
import { useNavigate } from 'react-router'
import { PlusIcon } from 'lucide-react'

export default function LandingPage() {
  const navigate = useNavigate()
  const login = useAuthStore(state => state.login)
  const isAuthed = useAuthStore(state => state.isAuthenticated)
  const isHydrated = useAuthStore(state => state.isHydrated)

  const onLoginClicked = async () => {
    try {
      await login()
      toast.success('Login Successfull')
    } catch (reason: any) {
      toast.error(reason)
    }
  }

  if (!isHydrated) {
    return <LoadingPage />
  }

  if (!isAuthed) {
    return (
      <>
        {/* <Navbar /> */}
        <main className='container mx-auto p-8 text-center'>
          <h1 className='text-4xl font-bold mb-4'>Write. Read. Inspire.</h1>
          <p className='mb-6 text-lg'>Share your stories and discover new voices—powered by ICP.</p>
          <Button onClick={onLoginClicked}>Internet Identity Login</Button>
        </main>
      </>
    )
  }

  return (
    <>
      {/* <Navbar showProfile /> */}
      <main className='container mx-auto p-8 space-y-6'>
        <h2 className='text-2xl font-semibold'>Recommended for you</h2>
        <p>YOHOHOHOHOHOHOHOHO</p>
        <Button onClick={() => navigate('/new-story')}>
          <PlusIcon /> New Story
        </Button>
      </main>
    </>
  )
}
