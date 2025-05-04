import { Link } from 'react-router'
import { Avatar, AvatarFallback, AvatarImage } from '../ui/avatar'

export function ProfileButton({ name }: { name?: string | null }) {
  return (
    <div className='flex items-center'>
      <Link to='/me'>
        <Avatar className='size-12 border-primary border-2'>
          <AvatarImage src={name ? `https://avatar.iran.liara.run/public?username=${name}` : undefined} />
          <AvatarFallback>{name ?? '?'}</AvatarFallback>
        </Avatar>
      </Link>
    </div>
  )
}
