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
  UNORDERED_LIST
} from '@lexical/markdown'
import lexicalTheme from '../themes/lexical'
import { EditorFloatingMenu } from './editor-floating-menu'

const initialConfig: InitialConfigType = {
  namespace: 'toic-story',
  theme: lexicalTheme,
  onError(error: Error) {
    throw error
  },
  nodes: [ListNode, ListItemNode, QuoteNode, HeadingNode, HorizontalRuleNode, LinkNode]
}

const transformers: Transformer[] = [
  HEADING,
  LINK,
  BOLD_ITALIC_UNDERSCORE,
  BOLD_STAR,
  BOLD_UNDERSCORE,
  ITALIC_STAR,
  ITALIC_UNDERSCORE,
  UNORDERED_LIST,
  QUOTE
]

type StoryEditorProps = {
  onChange: (editorState: EditorState, editor: LexicalEditor, tags: Set<string>) => void
}

function StoryEditor({ onChange }: StoryEditorProps) {
  return (
    <LexicalComposer initialConfig={initialConfig}>
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
        <OnChangePlugin onChange={onChange} />
      </div>
    </LexicalComposer>
  )
}

export { StoryEditor }
