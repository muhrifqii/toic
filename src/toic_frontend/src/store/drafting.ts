import { mapFromCategory, mapToCategory, optionOf, unwrapOption, unwrapResult } from '@/lib/mapper'
import { decodeId, encodeId } from '@/lib/string'
import { CandidOption } from '@/types/candid'
import { CategoryName } from '@/types/core'
import { toic_backend } from '@declarations/toic_backend'
import { SaveDraftArgs, StoryDetail } from '@declarations/toic_backend/toic_backend.did'
import { create } from 'zustand'

type DraftingState = {
  selectedId: string | null
  saving: boolean
  fetching: boolean
  draftTitle: string | null
  draftContent: string | null
  category: CategoryName | null
  description: string | null
}

type DraftingAction = {
  setActiveDraft: (id?: string | null) => void
  setCategory: (cat: CategoryName) => void
  setDraftTitle: (title?: string) => void
  setContent: (content?: string) => void
  save: (args: Partial<Pick<DraftingState, 'category' | 'description'>>) => void
  getDraft: (id: string) => void
  publish: () => Promise<string>
}

const initialState: DraftingState = {
  selectedId: null,
  saving: false,
  fetching: false,
  draftTitle: null,
  draftContent: null,
  category: null,
  description: null
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

  setDraftTitle: draftTitle => set({ draftTitle }),

  setContent: draftContent => {
    if (draftContent != null) {
      set({ draftContent })
      return
    }
    set({ draftContent: null })
  },

  /**
   * @throws error string
   */
  save: async ({ category, description }) => {
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
    const draftTitle = get().draftTitle
    const draftContent = get().draftContent
    const saveArgs: SaveDraftArgs = {
      title: optionOf(draftTitle),
      content: optionOf(draftContent),
      detail: storyDetailDid
    }

    if (selectedId === NewStoryIdPlaceholder) {
      const result = await toic_backend.create_draft(saveArgs)
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
      return set({ selectedId: encodeId(draft.id), saving: false, draftTitle, draftContent, category, description })
    }
    const actualId = decodeId(selectedId)
    const result = await toic_backend.update_draft(actualId, saveArgs)
    const [, err] = unwrapResult(result)
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
      description: hasDetailUpdate && description != null ? description : state.description
    }))
  },

  /**
   * @throws error string
   */
  getDraft: async (id: string) => {
    set({ fetching: true })
    const actualId = decodeId(id)
    const result = await toic_backend.get_draft(actualId)
    const [draft, err] = unwrapResult(result)
    set({ fetching: false })

    if (!!err) {
      throw err.message
    }
    if (!draft) {
      throw DraftNotFoundErrMsg
    }
    const [outline, content] = draft
    const storyDetail = unwrapOption(outline.detail)
    const draftTitle = outline.title

    const catDid = storyDetail?.category ?? null
    const category = catDid ? mapFromCategory(catDid) : null
    const description = storyDetail?.description ?? null
    const draftContent = content.content

    set({ description, category, draftContent, draftTitle, selectedId: id })
  },

  /**
   * @throws error string
   */
  publish: async () => {
    const id = get().selectedId
    if (!id) {
      console.error('publishing an empty id draft')
      throw 'Publish failed'
    }
    set({ saving: true })

    const actualId = decodeId(id)
    const result = await toic_backend.publish_draft(actualId)
    const [story, err] = unwrapResult(result)
    set({ saving: false })
    if (!!err) {
      throw err.message
    }
    set({ saving: false })
    if (!story) {
      console.log('null story after publish')
      throw 'Publish failed'
    }
    return encodeId(story.id)
  }
}))
