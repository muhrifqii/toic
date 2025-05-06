import { encodeId, formatDate } from '@/lib/string'
import { useFeedStore } from '@/store/feed'
import { StoryDetail } from '@declarations/toic_backend/toic_backend.did'
import { Dot, Plus, Share } from 'lucide-react'
import { useEffect } from 'react'
import { Link } from 'react-router'
import { StorySkeleton } from '../blocks/story-skeleton'
import { Button } from '../ui/button'
import { useAuthStore } from '@/store/auth'
import { Separator } from '../ui/separator'
import { Avatar, AvatarFallback, AvatarImage } from '../ui/avatar'
import { mapFromCategory, unwrapOption } from '@/lib/mapper'
import { Badge } from '../ui/badge'
import { CategoryName } from '@/types/core'

export default function HomeLayout() {
  const getRecommended = useFeedStore(state => state.getRecommended)
  const recommendedLoading = useFeedStore(state => state.fetchingRecommended)
  const recommendedStories = useFeedStore(state => state.recommended)

  const user = useAuthStore(state => state.user)

  useEffect(() => {
    getRecommended()
  }, [])

  return (
    <div className='container mx-auto p-8 space-y-6 max-w-4xl px-4 py-10'>
      <section>
        <h1 className='text-3xl font-bold'>
          Welcome back, <span className='text-primary-foreground'>{user?.name}</span>
        </h1>
        <p className='text-muted-foreground'>Your creative journey continues — keep writing and exploring.</p>
      </section>

      {/* Quick Actions */}
      <section className='flex gap-4 flex-wrap'>
        <Button asChild>
          <Link to='/new-story'>
            <Plus /> Write New Story
          </Link>
        </Button>
        <Button variant='outline'>
          <Share /> Share Your Profile
        </Button>
        {/* {tokenBalance > 1_000_000 && (
          <span className='rounded-full bg-green-100 text-green-700 px-3 py-1 text-sm'>AI Features Unlocked</span>
        )} */}
      </section>

      <Separator className='mt-8' />

      {/* Recommended Feed */}
      <section className='flex flex-col'>
        {recommendedLoading ? (
          <StorySkeleton />
        ) : recommendedStories.length === 0 ? (
          <div className='text-muted-foreground text-center'>It's empty in here</div>
        ) : (
          recommendedStories
            .map(p => ({
              title: p.title,
              id: encodeId(p.id),
              detail: p.detail,
              date: p.created_at,
              readTime: p.read_time,
              author: unwrapOption(p.author_name)
            }))
            .map(story => <RowItemContent {...story} key={story.id} />)
        )}
      </section>

      {/* Drafts Preview */}
      {/* {drafts.length > 0 && (
        <section>
          <h2 className='text-xl font-semibold mb-2'>Your Drafts</h2>
          <ul className='space-y-3'>
            {drafts.map(d => (
              <li key={d.id}>
                <Link to={`/edit/${encodeId(d.id)}`} className='text-blue-600 underline'>
                  {d.title || 'Untitled Draft'}
                </Link>
              </li>
            ))}
          </ul>
        </section>
      )} */}

      {/* Published Stories */}
      {/* {published.length > 0 && (
        <section>
          <h2 className='text-xl font-semibold mb-2'>Your Stories</h2>
          <ul className='space-y-3'>
            {published.map(p => (
              <li key={p.id}>
                <Link to={`/p/${encodeId(p.id)}`} className='text-blue-600 underline'>
                  {p.title}
                </Link>
              </li>
            ))}
          </ul>
        </section>
      )} */}

      {/* Token Summary / Referral */}
      {/* <section className='bg-muted p-4 rounded-lg shadow-sm'>
        <p className='text-muted-foreground'>
          You currently hold <span className='font-semibold'>{tokenBalance.toLocaleString()} TOIC</span>.
          {tokenBalance >= 1_000_000
            ? ' You have unlocked AI features.'
            : ' Stake 1,000,000 TOIC to unlock advanced writing tools.'}
        </p>
        <p className='mt-2 text-muted-foreground'>Invite friends using your referral code and earn airdrops!</p>
      </section> */}
    </div>
  )
}

type RowItemContentProp = {
  id: string
  title: string
  date: bigint
  readTime: number
  detail: StoryDetail
  author: string | null
}

function RowItemContent(prop: RowItemContentProp) {
  const fmtDate = formatDate(prop.date)
  return (
    <Link
      to={`/p/${prop.id}`}
      className='flex flex-row items-center justify-between border-b py-6 text-sm leading-tight last:border-b-0 w-full'
    >
      <div className='flex flex-col items-start gap-2'>
        <span className='line-clamp-2 font-semibold text-2xl'>{prop.title}</span>
        <span className='line-clamp-2 whitespace-break-spaces text-lg text-muted-foreground'>
          {prop.detail?.description}
        </span>
        <div className='flex w-full items-center gap-2'>
          <Badge className='mr-1'>{mapFromCategory(prop.detail.category)}</Badge>
          <span className='text-xs'>{prop.readTime} minute read</span>
          <Dot className='size-4' />
          <span className='text-xs'>{fmtDate}</span>
        </div>
      </div>
      <div className='flex flex-col items-center gap-2'>
        <Avatar className='size-10 border-1 border-primary'>
          <AvatarImage
            src={prop.author ? `https://avatar.iran.liara.run/public?username=${prop.author}` : undefined}
            alt={`${prop.author}'s avatar`}
          />
          <AvatarFallback className='text-4xl font-bold text-primary bg-primary-foreground'>
            {prop.author?.substring(0, 2)?.toUpperCase() ?? '??'}
          </AvatarFallback>
        </Avatar>
        <span className='text-xs text-foreground'>{prop.author}</span>
      </div>
    </Link>
  )
}
