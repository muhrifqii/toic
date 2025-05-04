import { StoryEditor } from '@/components/blocks/text-editor'
import { Textarea } from '@/components/ui/textarea'

export default function NewStoryPage() {
  return (
    <div className='container'>
      <div className='flex-col'>
        <Textarea
          placeholder='Title'
          aria-placeholder='title'
          className='!font-medium !text-5xl !h-fit !border-none !shadow-none !px-0 focus-visible:!ring-0 !bg-none resize-none'
        />
        <div className='mt-4'>
          <StoryEditor onChange={() => {}} />
        </div>
      </div>
    </div>
  )
}
