import { Button } from '@/components/ui/button'
import { authService } from '@/lib/auth'
import { toast } from 'sonner'

const onLoginClicked = async () => {
  console.log('login clicked')
  const auth = await authService()
  try {
    console.log('logging in', await auth.isAuthenticated())
    await auth.login()
    console.log('login success')
  } catch (reason: any) {
    toast.error(reason)
  }
}

export default function LandingPage() {
  return <Button onClick={onLoginClicked}>Internet Identity Login</Button>
}
