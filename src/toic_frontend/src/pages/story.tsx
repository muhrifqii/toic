import { DonateDialog } from '@/components/blocks/donate-dialog'
import { StorySkeleton } from '@/components/blocks/story-skeleton'
import { StoryEditor } from '@/components/blocks/text-editor'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Label } from '@/components/ui/label'
import { Separator } from '@/components/ui/separator'
import { formatDate, tokenDisplay } from '@/lib/string'
import { useAuthStore } from '@/store/auth'
import { useFeedStore } from '@/store/feed'
import { CoinsIcon, Flag, HeartHandshake, Share2, ThumbsUp } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useDebounceCallback } from 'usehooks-ts'

export default function StoryPage() {
  const navigate = useNavigate()
  const params = useParams()
  const id = params['id']

  const authed = useAuthStore(state => state.isAuthenticated)
  const reset = useFeedStore(state => state.resetCurrent)
  const getStory = useFeedStore(state => state.getStory)
  const support = useFeedStore(state => state.support)
  const donate = useFeedStore(state => state.donate)
  const story = useFeedStore(state => state.currentStory)
  const fetching = useFeedStore(state => state.fetching)
  const error = useFeedStore(state => state.error)

  const [supportCount, setSupportCount] = useState(0)
  const [donationDialogOpen, setDonationDialogOpen] = useState(false)

  const supportDebounced = useDebounceCallback(support, 1200)

  useEffect(() => {
    return () => {
      reset()
    }
  }, [])

  useEffect(() => {
    if (supportCount <= 0) {
      return
    }
    const c = supportCount
    supportDebounced(c)
    setSupportCount(0)
  }, [supportCount])

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
      <DonateDialog open={donationDialogOpen} onClose={() => setDonationDialogOpen(false)} />
      {!fetching ? (
        <div className='flex flex-col gap-6'>
          <div className='flex flex-col gap-1'>
            <div className='flex flex-row items-center gap-2'>
              <Avatar className='size-6 border-1 border-primary'>
                <AvatarImage
                  src={story?.author ? `https://avatar.iran.liara.run/public?username=${story?.author}` : undefined}
                  alt={`${story?.author}'s avatar`}
                />
                <AvatarFallback className='text-sm text-primary'>
                  {story?.author?.substring(0, 2)?.toUpperCase() ?? '??'}
                </AvatarFallback>
              </Avatar>
              <span className='text-xs text-foreground'>{story?.author}</span>
            </div>
            <span className='font-medium text-5xl h-fit '>{story?.title}</span>
          </div>
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
          {authed && (
            <div className='flex items-center justify-between gap-4 py-4'>
              <div className='flex items-center gap-4'>
                <div className='flex gap-3'>
                  <Button
                    id='support'
                    variant='ghost'
                    className='rounded-full'
                    onClick={() => setSupportCount(prev => prev + 1)}
                  >
                    <ThumbsUp className='size-5' />
                  </Button>
                  <Label>{story?.totalSupport ?? '-'}</Label>
                </div>
                <div className='flex gap-3'>
                  <Button id='tip' variant='ghost' className='rounded-full' onClick={() => setDonationDialogOpen(true)}>
                    <HeartHandshake className='size-5' />
                  </Button>
                  <Label>{tokenDisplay(story?.totalSupport)}</Label>
                </div>
              </div>
              <div className='flex items-center gap-2'>
                <Button
                  variant='ghost'
                  className='rounded-full'
                  onClick={async () => {
                    try {
                      await navigator.clipboard.writeText(window.location.href)
                      toast.success('Copied to clipboard!')
                    } catch (err) {
                      toast.error('Failed to share story!')
                    }
                  }}
                >
                  <Share2 className='size-5' />
                </Button>
              </div>
            </div>
          )}
        </div>
      ) : (
        <StorySkeleton />
      )}
    </div>
  )
}
