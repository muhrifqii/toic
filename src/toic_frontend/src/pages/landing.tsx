import { Button } from '@/components/ui/button'
import { authService } from '@/services/auth'
import { useAuthStore } from '@/store/auth'
import { toast } from 'sonner'

export default function LandingPage() {
  const login = useAuthStore(state => state.login)

  const onLoginClicked = async () => {
    try {
      await login()
      console.log('login success')
    } catch (reason: any) {
      toast.error(reason)
    }
  }
  return <Button onClick={onLoginClicked}>Internet Identity Login</Button>
}
