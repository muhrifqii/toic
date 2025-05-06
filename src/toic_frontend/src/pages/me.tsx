import { StorySkeleton } from '@/components/blocks/story-skeleton'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { mapFromCategory, unwrapOption } from '@/lib/mapper'
import { usePersonalStore } from '@/store/personal'
import { StoryDetail } from '@declarations/toic_backend/toic_backend.did'
import { useEffect } from 'react'
import { format } from 'date-fns'
import { Link } from 'react-router'
import { encodeId, formatDate, tokenDisplay } from '@/lib/string'
import { Dot } from 'lucide-react'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { useAuthStore } from '@/store/auth'
import { useWalletStore } from '@/store/wallet'
import { Badge } from '@/components/ui/badge'
import { Separator } from '@/components/ui/separator'

type TabMenu = 'published' | 'draft' | 'supported'

export default function MePage() {
  const resetStore = usePersonalStore(state => state.reset)

  const getDrafts = usePersonalStore(state => state.getDrafts)
  const getPublished = usePersonalStore(state => state.getPublished)

  const drafts = usePersonalStore(state => state.drafts)
  const fetchingDrafts = usePersonalStore(state => state.fetchingDrafts)
  const published = usePersonalStore(state => state.published)
  const fetchingPublished = usePersonalStore(state => state.fetchingPublished)

  useEffect(() => {
    getDrafts()
    getPublished()
    usePersonalStore.subscribe((state, prev) => {
      console.log('state changed', state)
    })
    return () => {
      resetStore()
    }
  }, [])

  return (
    <div>
      <ProfileView />
      <Tabs defaultValue='published' className='container flex flex-col'>
        <TabsList className='flex'>
          <TabsTrigger value='published' className='w-24'>
            Published
          </TabsTrigger>
          <TabsTrigger value='draft' className='w-24'>
            Draft
          </TabsTrigger>
          {/* <TabsTrigger value='supported' className='w-24'>
          Supported
        </TabsTrigger> */}
        </TabsList>
        <Content
          menu='published'
          loading={fetchingPublished}
          items={published.map(p => ({
            title: p.title,
            id: encodeId(p.id),
            detail: p.detail,
            date: p.created_at,
            readTime: p.read_time
          }))}
        />
        <Content
          menu='draft'
          loading={fetchingDrafts}
          items={drafts.map(d => ({
            title: d.title,
            id: encodeId(d.id),
            detail: unwrapOption(d.detail),
            date: unwrapOption(d.updated_at) ?? d.created_at,
            readTime: d.read_time
          }))}
        />
      </Tabs>
    </div>
  )
}

function ProfileView() {
  const user = useAuthStore(state => state.user)
  const name = unwrapOption(user?.name)
  const getBalance = useWalletStore(state => state.getBalance)
  const balance = useWalletStore(state => state.token)
  const getLockedBalance = useWalletStore(state => state.getLockedBalance)
  const staked = useWalletStore(state => state.lockedToken)

  useEffect(() => {
    getBalance()
    getLockedBalance()
  }, [])

  return (
    <div className='flex flex-col items-center bg-gradient-to-b from-primary to-primary/90 p-8 gap-4 rounded-b-2xl shadow-md'>
      <Avatar className='size-40 border-4 border-primary-foreground shadow'>
        <AvatarImage
          src={name ? `https://avatar.iran.liara.run/public?username=${name}` : undefined}
          alt={`${name}'s avatar`}
        />
        <AvatarFallback className='text-4xl font-bold text-primary bg-primary-foreground'>
          {name?.substring(0, 2)?.toUpperCase() ?? '??'}
        </AvatarFallback>
      </Avatar>

      <div className='text-3xl font-semibold text-primary-foreground'>{name}</div>

      <div className='flex items-center gap-4 bg-background px-6 py-2 rounded-full shadow-inner'>
        <div className='flex items-center gap-2'>
          <img src='/toic_token.png' alt='Token' className='size-6' />
          <span className='text-lg font-semibold text-primary'>{tokenDisplay(balance)}</span>
        </div>

        <Separator orientation='vertical' className='!h-8' />

        <div className='flex items-center gap-2 opacity-70'>
          <img src='/toic_token.png' alt='Staked Token' className='size-6 grayscale' />
          <span className='text-lg font-semibold text-muted-foreground'>{tokenDisplay(staked)}</span>
        </div>
      </div>
    </div>
  )
}

type RowItemContentProp = {
  menu: TabMenu
  id: string
  title: string
  date: bigint
  readTime: number
  detail: StoryDetail | null
}

function RowItemContent(prop: RowItemContentProp) {
  const fmtDate = formatDate(prop.date)
  const dateText =
    prop.menu === 'draft'
      ? `Last edited on ${fmtDate}`
      : prop.menu === 'published'
        ? `Published on ${fmtDate}`
        : fmtDate
  const dest = prop.menu === 'draft' ? `/x/${prop.id}/edit` : prop.menu === 'published' ? `/p/${prop.id}` : ``
  return (
    <Link
      to={dest}
      className='flex flex-row items-center justify-between border-b py-6 text-sm leading-tight last:border-b-0 w-full'
    >
      <div className='flex flex-col items-start gap-2'>
        <span className='line-clamp-2 font-semibold text-2xl'>{prop.title}</span>
        <span className='line-clamp-2 whitespace-break-spaces text-lg text-muted-foreground'>
          {prop.detail?.description}
        </span>
        <div className='flex w-full items-center gap-2'>
          {prop.detail?.category && <Badge className='mr-1'>{mapFromCategory(prop.detail.category)}</Badge>}
          <span className='text-xs'>{prop.readTime} minute read</span>
          <Dot className='size-4' />
          <span className='text-xs'>{dateText}</span>
        </div>
      </div>
    </Link>
  )
}

type ContentProps = {
  menu: TabMenu
  loading: boolean
  items: Omit<RowItemContentProp, 'menu'>[]
}

function Content({ menu, items, loading }: ContentProps) {
  const isEmpty = items.length === 0

  return (
    <TabsContent value={menu} className='flex flex-col'>
      {loading ? (
        <StorySkeleton />
      ) : isEmpty ? (
        <div className='flex text-3xl text-muted-foreground items-center h-40'>
          <h1 className='text-center w-full my-auto'>It's empty in here</h1>
        </div>
      ) : (
        items.map(item => <RowItemContent {...item} menu={menu} key={item.id} />)
      )}
    </TabsContent>
  )
}
