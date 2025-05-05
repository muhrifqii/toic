import { Link, useNavigate } from 'react-router'
import { Avatar, AvatarFallback, AvatarImage } from '../ui/avatar'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger
} from '../ui/dropdown-menu'
import { BadgeCheck, ChevronDown, CreditCard, LogOut, Sparkles } from 'lucide-react'
import { toast } from 'sonner'
import { StakeDialog } from './stake-dialog'
import { useState } from 'react'
import { useAuthStore } from '@/store/auth'

export function ProfileButton({ name }: { name?: string | null }) {
  const navigate = useNavigate()
  const [openStakeDialog, setOpenStakeDialog] = useState(false)
  const logout = useAuthStore(state => state.logout)

  return (
    <div className='flex items-center'>
      <StakeDialog open={openStakeDialog} onClose={() => setOpenStakeDialog(false)} />
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <div className='flex flex-row cursor-pointer items-center'>
            <Avatar className='size-12 rounded-full border-primary border-2'>
              <AvatarImage src={name ? `https://avatar.iran.liara.run/public?username=${name}` : undefined} />
              <AvatarFallback className='font-bold text-primary'>
                {name?.substring(0, 2)?.toUpperCase() ?? '??'}
              </AvatarFallback>{' '}
            </Avatar>
            <ChevronDown className='ml-2 size-4 text-primary' />
          </div>
        </DropdownMenuTrigger>
        <DropdownMenuContent
          className='w-[--radix-dropdown-menu-trigger-width] min-w-56 rounded-lg'
          side='bottom'
          align='end'
          sideOffset={4}
        >
          <DropdownMenuLabel className='p-0 font-normal'>
            <div className='flex items-center gap-2 px-1 py-1.5 text-left text-sm'>
              <Avatar className='size-12 rounded-full border-black dark:border-white border-2'>
                <AvatarImage src={name ? `https://avatar.iran.liara.run/public?username=${name}` : undefined} />
                <AvatarFallback className='font-bold text-primary'>
                  {name?.substring(0, 2)?.toUpperCase() ?? '??'}
                </AvatarFallback>{' '}
              </Avatar>
              <div className='grid flex-1 text-left text-sm leading-tight'>
                <span className='truncate font-semibold'>{name}</span>
              </div>
            </div>
          </DropdownMenuLabel>
          <DropdownMenuSeparator />
          <DropdownMenuGroup>
            <DropdownMenuItem onClick={() => setOpenStakeDialog(true)}>
              <Sparkles />
              Stake TOIC
            </DropdownMenuItem>
          </DropdownMenuGroup>
          <DropdownMenuSeparator />
          <DropdownMenuGroup>
            <DropdownMenuItem onClick={() => navigate('/me')}>
              <BadgeCheck />
              Account
            </DropdownMenuItem>
            <DropdownMenuItem onClick={() => toast.info('ICRC-3 comming soon')}>
              <CreditCard />
              Wallet
            </DropdownMenuItem>
          </DropdownMenuGroup>
          <DropdownMenuSeparator />
          <DropdownMenuItem onClick={logout}>
            <LogOut />
            Log out
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  )
}
