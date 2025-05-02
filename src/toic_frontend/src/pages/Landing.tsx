import { Button } from '@/components/ui/button'
import { authService } from '@/lib/auth'
import { toast } from 'sonner'

const onLoginClicked = async () => {
  const auth = await authService()
  try {
    await auth.login()
  } catch (reason: any) {
    toast.error(reason)
  }
}

export default function LandingPage() {
  return <Button onClick={onLoginClicked}>Internet Identity Login</Button>
}
