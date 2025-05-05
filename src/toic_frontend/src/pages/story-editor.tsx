import { NavbarEditor } from '@/components/blocks/navbar-editor'
import { StorySkeleton } from '@/components/blocks/story-skeleton'
import { StoryEditor } from '@/components/blocks/text-editor'
import { LoadingButton } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Skeleton } from '@/components/ui/skeleton'
import { Textarea } from '@/components/ui/textarea'
import { NewStoryIdPlaceholder, useDraftingStore } from '@/store/drafting'
import { useEffect, useLayoutEffect } from 'react'
import { useParams } from 'react-router'
import { useDebounceCallback } from 'usehooks-ts'

export default function StoryEditorPage() {
  const params = useParams()

  const setActiveDraft = useDraftingStore(state => state.setActiveDraft)
  const getDraft = useDraftingStore(state => state.getDraft)
  const setTitle = useDraftingStore(state => state.setDraftTitle)
  const setContent = useDraftingStore(state => state.setContent)
  const fetching = useDraftingStore(state => state.fetching)
  const saving = useDraftingStore(state => state.saving)
  const save = useDraftingStore(state => state.save)
  const publish = useDraftingStore(state => state.publish)
  const id = params['id']

  const setTitleDebounced = useDebounceCallback(setTitle, 1000)

  useEffect(() => {
    console.log('trigger param id changed', id)
    if (id) {
      getDraft(id)
    } else {
      setActiveDraft(NewStoryIdPlaceholder)
    }

    return () => {
      setActiveDraft(null)
    }
  }, [id])

  useEffect(() => {
    useDraftingStore.subscribe((state, prev) => {
      console.log('state changed', state)
    })
  }, [])

  return (
    <>
      <NavbarEditor>
        <div className='flex flex-row h-full py-6 justify-stretch items-center min-w-40 gap-4'>
          <LoadingButton isLoading={fetching} loadingText='Saving'>
            Save Draft
          </LoadingButton>
          <LoadingButton isLoading={fetching} loadingText='Publishing'>
            Publish
          </LoadingButton>
          <Separator orientation='vertical' />
        </div>
      </NavbarEditor>
      <div className='container'>
        {!fetching ? (
          <div className='flex-col'>
            <Textarea
              placeholder='Title'
              aria-placeholder='title'
              className='!font-medium !text-5xl !h-fit !border-none !rounded-none !shadow-none !px-0 focus-visible:!ring-0 !bg-none resize-none'
              onChange={e => setTitleDebounced(e.target.value)}
            />
            <div className='mt-4'>
              <StoryEditor onChange={setContent} />
            </div>
          </div>
        ) : (
          <StorySkeleton />
        )}
      </div>
    </>
  )
}
