import { encodeId, formatDate } from '@/lib/string'
import { useFeedStore } from '@/store/feed'
import { StoryDetail } from '@declarations/toic_backend/toic_backend.did'
import { Dot } from 'lucide-react'
import { useEffect } from 'react'
import { Link } from 'react-router'
import { StorySkeleton } from '../blocks/story-skeleton'

export default function HomeLayout() {
  const getRecommended = useFeedStore(state => state.getRecommended)

  const recommendedLoading = useFeedStore(state => state.fetchingRecommended)
  const recommendedStories = useFeedStore(state => state.recommended)

  useEffect(() => {
    getRecommended()
  }, [])

  return (
    <div>
      {recommendedLoading ? (
        <StorySkeleton />
      ) : recommendedStories.length === 0 ? (
        <div className='flex text-3xl text-muted-foreground items-center h-40'>
          <h1 className='text-center w-full my-auto'>It's empty in here</h1>
        </div>
      ) : (
        recommendedStories
          .map(p => ({
            title: p.title,
            id: encodeId(p.id),
            detail: p.detail,
            date: p.created_at,
            readTime: p.read_time
          }))
          .map(story => <RowItemContent {...story} key={story.id} />)
      )}
    </div>
  )
}

type RowItemContentProp = {
  id: string
  title: string
  date: bigint
  readTime: number
  detail: StoryDetail | null
}

function RowItemContent(prop: RowItemContentProp) {
  const fmtDate = formatDate(prop.date)
  return (
    <Link
      to={`/p/${prop.id}`}
      className='flex flex-col items-start gap-2 whitespace-nowrap border-b py-6 text-sm leading-tight last:border-b-0'
    >
      <span className='line-clamp-2 font-semibold text-2xl'>{prop.title}</span>
      <span className='line-clamp-2 whitespace-break-spaces text-lg text-muted-foreground'>
        {prop.detail?.description}
      </span>
      <div className='flex w-full items-center gap-2'>
        <span className='text-xs'>{prop.readTime} minute read</span>
        <Dot className='size-4' />
        <span className='text-xs'>{fmtDate}</span>
      </div>
    </Link>
  )
}
