import { LexicalComposer, type InitialConfigType } from '@lexical/react/LexicalComposer'
import { RichTextPlugin } from '@lexical/react/LexicalRichTextPlugin'
import { ContentEditable } from '@lexical/react/LexicalContentEditable'
import { HistoryPlugin } from '@lexical/react/LexicalHistoryPlugin'
import { MarkdownShortcutPlugin } from '@lexical/react/LexicalMarkdownShortcutPlugin'
import { OnChangePlugin } from '@lexical/react/LexicalOnChangePlugin'
import { LexicalErrorBoundary } from '@lexical/react/LexicalErrorBoundary'
import { EditorState, LexicalEditor } from 'lexical'
import { ListNode, ListItemNode } from '@lexical/list'
import { QuoteNode, HeadingNode } from '@lexical/rich-text'
import { HorizontalRuleNode } from '@lexical/react/LexicalHorizontalRuleNode'
import { LinkNode } from '@lexical/link'
import { FloatingMenuPlugin } from 'lexical-floating-menu'
import {
  BOLD_ITALIC_UNDERSCORE,
  BOLD_STAR,
  BOLD_UNDERSCORE,
  HEADING,
  ITALIC_STAR,
  ITALIC_UNDERSCORE,
  LINK,
  QUOTE,
  Transformer,
  UNORDERED_LIST,
  $convertToMarkdownString,
  $convertFromMarkdownString,
  BOLD_ITALIC_STAR
} from '@lexical/markdown'
import lexicalTheme from '../themes/lexical'
import { EditorFloatingMenu } from './editor-floating-menu'
import { useDebounceCallback } from 'usehooks-ts'

const initialConfig: (initial?: string, editable?: boolean) => InitialConfigType = (initial, editable) => ({
  namespace: 'toic-story',
  theme: lexicalTheme,
  onError(error: Error) {
    throw error
  },
  editable,
  nodes: [ListNode, ListItemNode, QuoteNode, HeadingNode, HorizontalRuleNode, LinkNode],
  editorState: initial ? () => $convertFromMarkdownString(initial, transformers) : undefined
})

const transformers: Transformer[] = [
  HEADING,
  LINK,
  BOLD_ITALIC_UNDERSCORE,
  BOLD_ITALIC_STAR,
  BOLD_STAR,
  BOLD_UNDERSCORE,
  ITALIC_STAR,
  ITALIC_UNDERSCORE,
  UNORDERED_LIST,
  QUOTE
]

type StoryEditorProps = {
  onChange?: (text: string) => void
  initialMd?: string
  editable?: boolean
}

function StoryEditor({ onChange, initialMd, editable = true }: StoryEditorProps) {
  const onChangeListener = useDebounceCallback((state: EditorState) => {
    state.read(() => {
      const mds = $convertToMarkdownString(transformers)
      onChange?.(mds)
    })
  }, 1000)

  return (
    <LexicalComposer initialConfig={initialConfig(initialMd, editable)}>
      <div className='relative'>
        <RichTextPlugin
          contentEditable={
            <ContentEditable className='prose max-w-full outline-none focus:outline-none text-base bg-background' />
          }
          aria-placeholder='Start writing your story…'
          placeholder={
            <div className='text-muted-foreground pointer-events-none absolute top-0'>
              <p>Start writing your story…</p>
            </div>
          }
          ErrorBoundary={LexicalErrorBoundary}
        />
        <HistoryPlugin />
        <MarkdownShortcutPlugin transformers={transformers} />
        <FloatingMenuPlugin MenuComponent={EditorFloatingMenu} element={document.body} />
        <OnChangePlugin onChange={(state, editor) => onChangeListener(state)} ignoreSelectionChange />
      </div>
    </LexicalComposer>
  )
}

export { StoryEditor }
