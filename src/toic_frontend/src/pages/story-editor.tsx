import { NavbarEditor } from '@/components/blocks/navbar-editor'
import { StorySkeleton } from '@/components/blocks/story-skeleton'
import { StoryEditor } from '@/components/blocks/text-editor'
import { Button, LoadingButton } from '@/components/ui/button'
import { Form, FormControl, FormField, FormItem, FormLabel, FormMessage } from '@/components/ui/form'
import { Input } from '@/components/ui/input'
import { Separator } from '@/components/ui/separator'
import {
  Sheet,
  SheetClose,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
  SheetTrigger
} from '@/components/ui/sheet'
import { Textarea } from '@/components/ui/textarea'
import { publishSchema, PublishSingValues } from '@/lib/validations/publish'
import RouteEditorGuard from '@/routes/editor-guard'
import { NewStoryIdPlaceholder, useDraftingStore } from '@/store/drafting'
import { CategoryName, categoryNames } from '@/types/core'
import { zodResolver } from '@hookform/resolvers/zod'
import { useEffect } from 'react'
import { useForm } from 'react-hook-form'
import { useNavigate, useParams } from 'react-router'
import { toast } from 'sonner'
import { useDebounceCallback } from 'usehooks-ts'

export default function StoryEditorPage() {
  const navigate = useNavigate()

  const selectedId = useDraftingStore(state => state.selectedId)
  const setTitle = useDraftingStore(state => state.setDraftTitle)
  const setContent = useDraftingStore(state => state.setContent)
  const fetching = useDraftingStore(state => state.fetching)
  const title = useDraftingStore(state => state.draftTitle)
  const content = useDraftingStore(state => state.draftContent)

  const setTitleDebounced = useDebounceCallback(setTitle, 1000)

  useEffect(() => {
    if (!selectedId || selectedId === NewStoryIdPlaceholder) {
      return
    }
    navigate(`/x/${selectedId}/edit`, { replace: true })
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
      <NavbarArea />
      <div className='container'>
        {!fetching ? (
          <div className='flex-col'>
            <Textarea
              placeholder='Title'
              aria-placeholder='title'
              className='!font-medium !text-5xl !h-fit !border-none !rounded-none !shadow-none !px-0 focus-visible:!ring-0 !bg-none resize-none'
              onChange={e => setTitleDebounced(e.target.value)}
              defaultValue={title ?? undefined}
            />
            <div className='mt-4'>
              <StoryEditor onChange={setContent} initialMd={content ?? undefined} />
            </div>
          </div>
        ) : (
          <StorySkeleton />
        )}
      </div>
    </RouteEditorGuard>
  )
}

function NavbarArea() {
  const navigate = useNavigate()
  const setDetail = useDraftingStore(state => state.saveDetail)
  const detailDescription = useDraftingStore(state => state.description)
  const detailCategory = useDraftingStore(state => state.category)
  const selectedId = useDraftingStore(state => state.selectedId)
  const saving = useDraftingStore(state => state.saving)
  const fetching = useDraftingStore(state => state.fetching)
  const publishing = useDraftingStore(state => state.publishing)
  const publish = useDraftingStore(state => state.publish)
  const aiForDescription = useDraftingStore(state => state.assistAiDescription)
  const aiForGeneration = useDraftingStore(state => state.assistAiStory)
  const aiLoading = useDraftingStore(state => state.aiLoading)
  const error = useDraftingStore(state => state.error)
  const handleError = useDraftingStore(state => state.errorHandled)

  const form = useForm<PublishSingValues>({
    resolver: zodResolver(publishSchema),
    defaultValues: { description: detailDescription ?? '', category: detailCategory ?? undefined }
  })

  useEffect(() => {
    if (detailCategory == null && !detailDescription) return

    form.reset({ description: detailDescription ?? '', category: detailCategory ?? undefined })
  }, [detailDescription, detailCategory])

  useEffect(() => {
    if (error) {
      toast.error(error)
      handleError()
    }
  }, [error])

  const onSubmitPublish = async (val: PublishSingValues) => {
    await setDetail(val)
    await publish()
    navigate('/')
  }
  const onSheetOpenChanges = (open: boolean) => {
    if (!open) {
      setDetail(form.getValues())
    }
  }

  return (
    <NavbarEditor>
      <div className='flex flex-row h-full py-6 justify-stretch items-center gap-4'>
        <h1 className='text-muted-foreground'>
          {saving ? 'Saving' : selectedId !== NewStoryIdPlaceholder ? 'Saved' : ''}
        </h1>
        <Sheet onOpenChange={onSheetOpenChanges}>
          <SheetTrigger asChild>
            <LoadingButton
              loadingText='Publishing'
              disabled={fetching || saving || selectedId === NewStoryIdPlaceholder}
            >
              Publish
            </LoadingButton>
          </SheetTrigger>
          <Form {...form}>
            <form id='publish-form' onSubmit={form.handleSubmit(onSubmitPublish)}>
              <SheetContent className='flex flex-col' side='right'>
                <SheetHeader className='gap-1'>
                  <SheetTitle>Publish Your Story</SheetTitle>
                  <SheetDescription>Publish your story now and reach more audience</SheetDescription>
                </SheetHeader>
                <div className='flex flex-1 flex-col gap-8 overflow-y-auto p-4 text-sm'>
                  <FormField
                    control={form.control}
                    name='description'
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel htmlFor='description'>Story Description</FormLabel>
                        <FormControl>
                          <Textarea placeholder='Description' {...field} />
                        </FormControl>
                        <FormMessage />
                        <LoadingButton variant='secondary' onClick={aiForDescription} isLoading={aiLoading}>
                          Generate description using AI
                        </LoadingButton>
                      </FormItem>
                    )}
                  />

                  <FormField
                    control={form.control}
                    name='category'
                    render={({ field }) => (
                      <FormItem>
                        <FormLabel>Pick the category it belongs to</FormLabel>
                        <FormControl>
                          <div className='grid grid-cols-2 md:grid-cols-3 gap-3'>
                            {categoryNames.map(category => {
                              const isSelected = field.value === category
                              return (
                                <Button
                                  type='button'
                                  key={category}
                                  onClick={() => {
                                    field.onChange(category)
                                  }}
                                  className={`rounded-lg border p-3 text-sm font-medium transition-all ${
                                    isSelected
                                      ? 'bg-primary text-primary-foreground border-primary'
                                      : 'bg-muted text-muted-foreground hover:bg-muted/80 border-muted-foreground'
                                  }`}
                                >
                                  {category}
                                </Button>
                              )
                            })}
                          </div>
                        </FormControl>
                        <FormMessage />
                      </FormItem>
                    )}
                  />
                  <LoadingButton
                    variant='default'
                    onClick={() => toast.info('Feature comming soon!')}
                    isLoading={aiLoading}
                  >
                    Expand Writing using AI
                  </LoadingButton>
                </div>
                <SheetFooter className='mt-auto flex gap-2 flex-col space-x-0'>
                  <LoadingButton
                    form='publish-form'
                    type='submit'
                    className='w-full'
                    loadingText='Publishing'
                    isLoading={publishing || form.formState.isSubmitting}
                    disabled={!form.formState.isValid || aiLoading}
                  >
                    Publish Now
                  </LoadingButton>
                  <SheetClose asChild>
                    <Button variant='outline' className='w-full'>
                      Save
                    </Button>
                  </SheetClose>
                </SheetFooter>
              </SheetContent>
            </form>
          </Form>
        </Sheet>

        <Separator orientation='vertical' />
      </div>
    </NavbarEditor>
  )
}
