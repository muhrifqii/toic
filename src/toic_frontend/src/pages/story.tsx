import { StorySkeleton } from '@/components/blocks/story-skeleton'
import { StoryEditor } from '@/components/blocks/text-editor'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { formatDate, tokenDisplay } from '@/lib/string'
import { useFeedStore } from '@/store/feed'
import { CoinsIcon, Flag, HeartHandshake, Share2, ThumbsUp } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router'

export default function StoryPage() {
  const navigate = useNavigate()
  const params = useParams()
  const id = params['id']

  const reset = useFeedStore(state => state.resetCurrent)
  const getStory = useFeedStore(state => state.getStory)
  const story = useFeedStore(state => state.currentStory)
  const fetching = useFeedStore(state => state.fetching)
  const error = useFeedStore(state => state.error)

  useEffect(() => {
    return () => {
      reset()
    }
  }, [])

  useEffect(() => {
    if (id) {
      getStory(id)
    } else {
      navigate('/', { replace: true })
    }
  }, [id])

  useEffect(() => {
    if (error === '404') {
      navigate('/404', { replace: true })
    }
  }, [error])

  return (
    <div className='container flex flex-col gap-8'>
      {!fetching ? (
        <div className='flex flex-col gap-6'>
          <div className='font-medium text-5xl h-fit '>{story?.title}</div>
          <div className='flex flex-row gap-2 items-center text-muted-foreground h-8'>
            <Badge>{story?.category}</Badge>
            <Separator orientation='vertical' />
            <span>{story?.readTime} minute read</span>
            <Separator orientation='vertical' />
            <span>{formatDate(story?.createdAt ?? 0n)}</span>
          </div>
          <div className='mt-4 !select-none'>
            <StoryEditor initialMd={story?.content ?? ''} editable={false} />
          </div>
          <div className='flex items-center justify-between gap-4 py-4'>
            <div className='flex items-center gap-4'>
              <div className='flex gap-3'>
                <Button id='support' variant='ghost' className='rounded-full'>
                  <ThumbsUp className='size-5' />
                </Button>
                <Label>{story?.totalSupport ?? '-'}</Label>
              </div>
              <div className='flex gap-3'>
                <Button id='tip' variant='ghost' className='rounded-full'>
                  <HeartHandshake className='size-5' />
                </Button>
                <Label>{tokenDisplay(story?.totalSupport)}</Label>
              </div>
            </div>
            <div className='flex items-center gap-2'>
              <Button variant='ghost' className='rounded-full'>
                <Share2 className='size-5' />
              </Button>
            </div>
          </div>
        </div>
      ) : (
        <StorySkeleton />
      )}
    </div>
  )
}
