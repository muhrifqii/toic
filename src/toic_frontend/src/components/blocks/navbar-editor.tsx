import { Link } from 'react-router'
import { Button } from '@/components/ui/button'
import { useAuthStore } from '@/store/auth'
import { Label } from '@/components/ui/label'
import { ProfileButton } from './profile-button'
import { unwrapOption } from '@/lib/mapper'

export function NavbarEditor() {
  const isAuthed = useAuthStore(state => state.isAuthenticated)
  const isHydrated = useAuthStore(state => state.isHydrated)
  const user = useAuthStore(state => state.user)
  const name = unwrapOption(user?.name)

  return (
    <header className='sticky top-0 z-50 w-full h-24 bg-sidebar backdrop-blur border-b'>
      <div className='flex max-w-4xl mx-auto items-center justify-between px-4 h-full !py-0'>
        <Link to='/' className='flex tracking-tight text-sidebar-foreground gap-4'>
          <img id='nav-app-logo' aria-label='toic logo' src='/toic_token.png' className='object-cover h-20' />
          <Label htmlFor='nav-app-logo' aria-label='TOIC' className='font-semibold text-3xl'>
            TOIC
          </Label>
        </Link>

        {isHydrated && (
          <div className='flex items-center gap-4'>
            <Link to='/new-story'>
              <Button>Publish</Button>
            </Link>
            <ProfileButton name={name} />
          </div>
        )}
      </div>
    </header>
  )
}
