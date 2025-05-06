import { Story, StoryContent } from '@declarations/toic_backend/toic_backend.did'
import { create } from 'zustand'
import { beService } from './auth'
import { decodeId, encodeId } from '@/lib/string'
import { mapFromCategory, unwrapOption, unwrapResult } from '@/lib/mapper'
import { CategoryName } from '@/types/core'

type StoryStorable = {
  title: string | null
  content: string | null
  description: string | null
  category: CategoryName | null
  readTime: number
  createdAt: bigint
  totalSupport: number
  totalTip: bigint
  author: string | null
}

type FeedState = {
  recommended: Story[]

  currentId: string | null
  currentStory: StoryStorable | null
  fetching: boolean
  fetchingRecommended: boolean
  error: '404' | '401' | 'api' | null
  errorMessage: string | null
  commonLoading: boolean
}

type FeedAction = {
  getRecommended: () => Promise<void>
  getStory: (id: string) => Promise<void>
  resetCurrent: () => void
  reset: () => void
  support: (n: number) => void
  donate: (amount: bigint) => void
}

const initialState: FeedState = {
  recommended: [],
  currentId: null,
  currentStory: null,
  fetching: false,
  fetchingRecommended: false,
  error: null,
  errorMessage: null,
  commonLoading: false
}

export const useFeedStore = create<FeedState & FeedAction>()((set, get) => ({
  ...initialState,
  getRecommended: async () => {
    set({ fetchingRecommended: true })
    // skip pagination for now
    const result = await beService().get_recommended_stories({ cursor: [], limit: [] })
    const [tuple, err] = unwrapResult(result)
    set({ fetchingRecommended: false })
    if (err) {
      console.error(err.message)
      return
    }
    if (!tuple) {
      return
    }
    const [, vec] = tuple
    set({ recommended: vec })
  },
  getStory: async (id: string) => {
    set({ fetching: true, error: null, errorMessage: null })
    const actualId = decodeId(id)
    const result = await beService().get_story(actualId)
    const [tuple, err] = unwrapResult(result)
    set({ fetching: false })

    if (err) {
      console.error(err.message)
      return
    }
    if (!tuple) {
      set({ error: '404' })
      return
    }
    const [outline, content] = tuple
    const story: StoryStorable = {
      title: outline.title,
      content: content.content,
      description: outline.detail.description,
      category: mapFromCategory(outline.detail.category),
      readTime: outline.read_time,
      createdAt: outline.created_at,
      totalSupport: outline.total_support,
      totalTip: outline.total_tip_support,
      author: unwrapOption(outline.author_name)
    }
    set({ currentStory: story, currentId: encodeId(outline.id) })
  },
  resetCurrent: () => {
    set(prev => ({ ...initialState, recommended: prev.recommended }))
  },
  reset: () => {
    set({ ...initialState })
  },

  support: async (n: number) => {
    const id = get().currentId
    if (id == null) {
      return
    }

    set({ error: null, errorMessage: null, commonLoading: true })

    const result = await beService().support_story({
      id: decodeId(id),
      tip: [],
      support: [n]
    })
    const [, err] = unwrapResult(result)
    set({ commonLoading: false })
    if (err) {
      console.error(err.message)
      set({ error: 'api', errorMessage: err.message })
      return
    }
    set(prev => ({ currentStory: { ...prev.currentStory!, totalSupport: prev.currentStory!.totalSupport + n } }))
  },

  donate: async (amount: bigint) => {
    const id = get().currentId
    if (id == null) {
      return
    }

    set({ error: null, errorMessage: null, commonLoading: true })

    const result = await beService().support_story({
      id: decodeId(id),
      tip: [amount],
      support: []
    })
    const [, err] = unwrapResult(result)
    set({ commonLoading: false })
    if (err) {
      console.error(err.message)
      set({ error: 'api', errorMessage: err.message })
      return
    }

    set(prev => ({ currentStory: { ...prev.currentStory!, totalTip: prev.currentStory!.totalTip + amount } }))
  }
}))
