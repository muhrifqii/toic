import { Story, StoryContent } from '@declarations/toic_backend/toic_backend.did'
import { create } from 'zustand'
import { beService } from './auth'
import { decodeId, encodeId } from '@/lib/string'
import { mapFromCategory, unwrapResult } from '@/lib/mapper'
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
}

type FeedState = {
  recommended: Story[]

  currentId: string | null
  currentStory: StoryStorable | null
  fetching: boolean
  fetchingRecommended: boolean
  error: '404' | null
}

type FeedAction = {
  getRecommended: () => Promise<void>
  getStory: (id: string) => Promise<void>
  resetCurrent: () => void
  reset: () => void
}

const initialState: FeedState = {
  recommended: [],
  currentId: null,
  currentStory: null,
  fetching: false,
  fetchingRecommended: false,
  error: null
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
    set({ fetching: true })
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
      totalTip: outline.total_tip_support
    }
    set({ currentStory: story, currentId: encodeId(outline.id) })
  },
  resetCurrent: () => {
    set(prev => ({ ...initialState, recommended: prev.recommended }))
  },
  reset: () => {
    set({ ...initialState })
  }
}))
