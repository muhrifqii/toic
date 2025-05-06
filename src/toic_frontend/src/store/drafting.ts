import { mapFromCategory, mapToCategory, optionOf, unwrapOption, unwrapResult } from '@/lib/mapper'
import { decodeId, encodeId } from '@/lib/string'
import { CandidOption } from '@/types/candid'
import { CategoryName } from '@/types/core'
import { SaveDraftArgs, StoryDetail } from '@declarations/toic_backend/toic_backend.did'
import { toast } from 'sonner'
import { create } from 'zustand'
import { beService } from './auth'

type DraftingState = {
  selectedId: string | null
  saving: boolean
  fetching: boolean
  publishing: boolean
  draftTitle: string | null
  draftContent: string | null
  category: CategoryName | null
  description: string | null
  error: '404' | string | null
  readTime: number
  aiLoading: boolean
}

type DraftingAction = {
  setActiveDraft: (id?: string | null) => void
  setCategory: (cat: CategoryName) => void
  setDraftTitle: (title?: string) => Promise<void>
  setContent: (content?: string) => Promise<void>
  save: (args: Partial<Pick<DraftingState, 'category' | 'description' | 'draftTitle' | 'draftContent'>>) => void
  saveDetail: (args: Partial<Pick<DraftingState, 'category' | 'description'>>) => Promise<void>
  getDraft: (id: string) => Promise<void>
  publish: () => Promise<string>
  assistAiDescription: () => void
  assistAiStory: () => void
  errorHandled: () => void
}

const initialState: DraftingState = {
  selectedId: null,
  saving: false,
  fetching: false,
  publishing: false,
  draftTitle: null,
  draftContent: null,
  category: null,
  description: null,
  error: null,
  readTime: 0,
  aiLoading: false
}

export const NewStoryIdPlaceholder = 'itisnewstory'
export const DraftNotFoundErrMsg = 'Draft not found'

export const useDraftingStore = create<DraftingState & DraftingAction>()((set, get) => ({
  ...initialState,

  setActiveDraft: async (id?: string | null) => {
    if (id) {
      set({ selectedId: id })
      return
    }
    set({ ...initialState })
  },

  setCategory: cat => set({ category: cat }),

  errorHandled: () => set({ error: null }),

  setDraftTitle: async draftTitle => {
    if (get().saving) {
      console.log('saving while having another saving in progress')
      return
    }
    const selectedId = get().selectedId
    if (!selectedId) {
      console.error('saving on a non id selected')
      throw 'Saving failed'
    }
    get().save({ draftTitle })
  },

  setContent: async draftContent => {
    if (get().saving) {
      console.log('saving while having another saving in progress')
      return
    }
    const selectedId = get().selectedId
    if (!selectedId) {
      console.error('saving on a non id selected')
      throw 'Saving failed'
    }

    get().save({ draftContent })
  },

  saveDetail: async ({ category, description }) => {
    if (get().saving) {
      console.log('saving while having another saving in progress')
      return
    }
    const selectedId = get().selectedId
    if (!selectedId) {
      console.error('saving on a non id selected')
      throw 'Saving failed'
    }

    let hasDetailUpdate = false
    // add extra validation inside
    if (category && get().category !== category) {
      hasDetailUpdate = true
    }
    if (description != null && get().description !== description) {
      hasDetailUpdate = true
    }

    if (!hasDetailUpdate) {
      console.log('no change, skipping')
      return
    }

    set({ saving: true })

    const storyDetailDid: CandidOption<StoryDetail> = hasDetailUpdate
      ? [
          {
            description: description ?? '',
            mature_content: false,
            category: mapToCategory(category ?? get().category ?? 'NonFiction')
          }
        ]
      : []

    const saveArgs: SaveDraftArgs = {
      title: [],
      content: [],
      detail: storyDetailDid
    }

    if (selectedId === NewStoryIdPlaceholder) {
      const result = await beService().create_draft(saveArgs)
      const [draft, err] = unwrapResult(result)
      if (err) {
        set({ saving: false })
        throw err.message
      }
      if (!draft) {
        console.log('null draft but no error returned')
        set({ saving: false })
        throw 'Saving failed'
      }
      return set({ selectedId: encodeId(draft.id), saving: false, category, description })
    }
    const actualId = decodeId(selectedId)
    const result = await beService().update_draft(actualId, saveArgs)
    const [read_time, err] = unwrapResult(result)
    if (err) {
      set({ saving: false })
      throw err.message
    }
    return set(state => ({
      saving: false,
      category: hasDetailUpdate && category ? category : state.category,
      description: hasDetailUpdate && description != null ? description : state.description,
      readTime: read_time ?? state.readTime
    }))
  },

  save: async ({ category, description, draftTitle, draftContent }) => {
    if (get().saving) {
      console.log('saving while having another saving in progress')
      return
    }
    const selectedId = get().selectedId
    if (!selectedId) {
      console.error('saving on a non id selected')
      throw 'Saving failed'
    }

    set({ saving: true })

    let hasDetailUpdate = false
    // add extra validation inside
    if (category && get().category !== category) {
      hasDetailUpdate = true
    }
    if (description != null && get().description !== description) {
      hasDetailUpdate = true
    }

    const storyDetailDid: CandidOption<StoryDetail> = hasDetailUpdate
      ? [
          {
            description: description ?? '',
            mature_content: false,
            category: mapToCategory(category ?? get().category ?? 'NonFiction')
          }
        ]
      : []
    const saveArgs: SaveDraftArgs = {
      title: optionOf(draftTitle),
      content: optionOf(draftContent),
      detail: storyDetailDid
    }

    if (selectedId === NewStoryIdPlaceholder) {
      const result = await beService().create_draft(saveArgs)
      const [draft, err] = unwrapResult(result)
      if (err) {
        set({ saving: false })
        throw err.message
      }
      if (!draft) {
        console.log('null draft but no error returned')
        set({ saving: false })
        throw 'Saving failed'
      }
      return set({
        selectedId: encodeId(draft.id),
        saving: false,
        draftTitle,
        draftContent,
        category,
        description,
        readTime: draft.read_time
      })
    }
    const actualId = decodeId(selectedId)
    const result = await beService().update_draft(actualId, saveArgs)
    const [read_time, err] = unwrapResult(result)
    if (err) {
      set({ saving: false })
      throw err.message
    }
    const titleUpdated = saveArgs.title.length > 0
    const contentUpdated = saveArgs.content.length > 0
    return set(state => ({
      saving: false,
      draftTitle: titleUpdated ? draftTitle : state.draftTitle,
      draftContent: contentUpdated ? draftContent : state.draftContent,
      category: hasDetailUpdate && category ? category : state.category,
      description: hasDetailUpdate && description != null ? description : state.description,
      readTime: read_time ?? state.readTime
    }))
  },

  getDraft: async (id: string) => {
    if (id === NewStoryIdPlaceholder) {
      console.warn('Prohibited action: getDraft')
      return
    }
    set({ fetching: true })
    const actualId = decodeId(id)
    const result = await beService().get_draft(actualId)
    const [draft, err] = unwrapResult(result)
    set({ fetching: false })

    if (!!err) {
      toast.error(err.message)
      set({ error: '404' })
      return
    }
    if (!draft) {
      toast.error(DraftNotFoundErrMsg)
      set({ error: '404' })
      return
    }

    const [outline, content] = draft
    const storyDetail = unwrapOption(outline.detail)
    const draftTitle = outline.title

    const catDid = storyDetail?.category ?? null
    const category = catDid ? mapFromCategory(catDid) : null
    const description = storyDetail?.description ?? null
    const draftContent = content.content

    set({ description, category, draftContent, draftTitle, selectedId: id, readTime: outline.read_time })
  },

  publish: async () => {
    const id = get().selectedId
    if (!id) {
      console.error('publishing an empty id draft')
      throw 'Publish failed'
    }
    set({ publishing: true })

    const actualId = decodeId(id)
    const result = await beService().publish_draft(actualId)
    const [story, err] = unwrapResult(result)
    set({ publishing: false })
    if (!!err) {
      throw err.message
    }
    set({ publishing: false })
    if (!story) {
      console.log('null story after publish')
      throw 'Publish failed'
    }
    return encodeId(story.id)
  },
  assistAiDescription: async () => {
    const id = get().selectedId
    if (!id) {
      console.error('id is empty')
      return
    }
    set({ aiLoading: true })
    const actualId = decodeId(id)
    const result = await beService().assist_action({ GenerateDescription: actualId })
    const [str, err] = unwrapResult(result)
    set({ aiLoading: false })
    if (!!err || !str) {
      console.error(err?.message ?? 'empty result')
      set({ error: err?.message ?? 'Cannot use this function at the moment' })
      return
    }
    let desc = str
    if (str.startsWith('\"')) {
      desc = str.substring(1, str.length - 1)
    }
    set({ description: desc })
  },
  assistAiStory: async () => {
    const id = get().selectedId
    if (!id) {
      console.error('id is empty')
      return
    }
    set({ aiLoading: true })
    const actualId = decodeId(id)
    const result = await beService().assist_action({ ExpandWriting: actualId })
    const [str, err] = unwrapResult(result)
    set({ aiLoading: true })
    if (!!err || !str) {
      console.error(err?.message ?? 'empty result')
      set({ error: err?.message ?? 'Cannot use this function at the moment' })
    }
    set(prev => ({ draftContent: prev.draftContent ?? '' + str }))
  }
}))
