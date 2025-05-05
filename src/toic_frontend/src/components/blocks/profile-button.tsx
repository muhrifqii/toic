import { Link } from 'react-router'
import { Avatar, AvatarFallback, AvatarImage } from '../ui/avatar'

export function ProfileButton({ name }: { name?: string | null }) {
  return (
    <div className='flex items-center'>
      <Link to='/me'>
        <Avatar className='size-12 border-primary border-2'>
          <AvatarImage src={name ? `https://avatar.iran.liara.run/public?username=${name}` : undefined} />
          <AvatarFallback className='font-bold text-primary'>
            {name?.substring(0, 2)?.toUpperCase() ?? '??'}
          </AvatarFallback>
        </Avatar>
      </Link>
    </div>
  )
}
