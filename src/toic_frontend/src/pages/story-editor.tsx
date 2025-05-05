import { NavbarEditor } from '@/components/blocks/navbar-editor'
import { StorySkeleton } from '@/components/blocks/story-skeleton'
import { StoryEditor } from '@/components/blocks/text-editor'
import { LoadingButton } from '@/components/ui/button'
import { Separator } from '@/components/ui/separator'
import { Textarea } from '@/components/ui/textarea'
import RouteEditorGuard from '@/routes/editor-guard'
import { NewStoryIdPlaceholder, useDraftingStore } from '@/store/drafting'
import { useEffect } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useDebounceCallback } from 'usehooks-ts'

export default function StoryEditorPage() {
  const navigate = useNavigate()

  const selectedId = useDraftingStore(state => state.selectedId)
  const setTitle = useDraftingStore(state => state.setDraftTitle)
  const setContent = useDraftingStore(state => state.setContent)
  const fetching = useDraftingStore(state => state.fetching)
  const saving = useDraftingStore(state => state.saving)
  const publish = useDraftingStore(state => state.publish)

  const setTitleDebounced = useDebounceCallback(setTitle, 1000)

  useEffect(() => {
    if (!selectedId || selectedId === NewStoryIdPlaceholder) {
      return
    }
    console.log('should navigate to ', selectedId)
    // navigate(`/x/${selectedId}/edit`, { replace: true })
  }, [selectedId])

  // delete below later
  useEffect(() => {
    useDraftingStore.subscribe((state, prev) => {
      console.log('state changed', state)
    })
  }, [])
  // delete above later

  return (
    <RouteEditorGuard>
      <NavbarEditor>
        <div className='flex flex-row h-full py-6 justify-stretch items-center gap-4'>
          {/* <LoadingButton isLoading={fetching || saving} loadingText='Saving'>
            Save Draft
          </LoadingButton> */}
          <LoadingButton
            isLoading={saving}
            loadingText='Publishing'
            disabled={fetching || selectedId === NewStoryIdPlaceholder}
          >
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
    </RouteEditorGuard>
  )
}
