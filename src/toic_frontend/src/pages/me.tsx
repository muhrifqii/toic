import { StorySkeleton } from '@/components/blocks/story-skeleton'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { unwrapOption } from '@/lib/mapper'
import { usePersonalStore } from '@/store/personal'
import { StoryDetail } from '@declarations/toic_backend/toic_backend.did'
import { useEffect } from 'react'
import { format } from 'date-fns'
import { Link } from 'react-router'
import { encodeId } from '@/lib/string'
import { Dot } from 'lucide-react'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { useAuthStore } from '@/store/auth'
import { useWalletStore } from '@/store/wallet'
import { Badge } from '@/components/ui/badge'

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
        <Content menu='published' loading={false} items={[]} />
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

  useEffect(() => {
    getBalance()
  }, [])

  return (
    <div className='flex flex-col items-center bg-primary to-primary p-6 gap-2'>
      <Avatar className='size-48 border-primary-foreground border-4'>
        <AvatarImage src={name ? `https://avatar.iran.liara.run/public?username=${name}` : undefined} />
        <AvatarFallback className='font-bold text-primary text-4xl'>
          {name?.substring(0, 2)?.toUpperCase() ?? '??'}
        </AvatarFallback>
      </Avatar>
      <div className='text-3xl text-primary-foreground font-medium'>{name}</div>
      <Badge className='flex flex-row items-center gap-1 text-primary-foreground bg-background px-4 cursor-default'>
        <img src='/toic_token.png' className='size-8' />
        <span className='text-xl font-semibold mr-2'>{balance ?? '0'}</span>
      </Badge>
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
  const fmtDate = format(Number(prop.date / 1_000_000n), 'dd MMM yyyy')
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
      className='flex flex-col items-start gap-2 whitespace-nowrap border-b py-6 text-sm leading-tight last:border-b-0'
    >
      <span className='line-clamp-2 font-medium text-base'>{prop.title}</span>
      <span className='line-clamp-2 w-[260px] whitespace-break-spaces text-xs'>{prop.detail?.description}</span>
      <div className='flex w-full items-center gap-2'>
        <span className='text-xs'>{prop.readTime} minute read</span>
        <Dot className='size-4' />
        <span className='text-xs'>{dateText}</span>
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
