import { useEffect, useState, ReactNode, useCallback } from 'react'
import { $getSelection, $isRangeSelection, FORMAT_TEXT_COMMAND, TextFormatType } from 'lexical'
import { FloatingMenuComponentProps } from 'lexical-floating-menu'
import { Toggle } from '@/components/ui/toggle'
import { Bold, Italic } from 'lucide-react'
import { ToggleGroup } from '@/components/ui/toggle-group'

export type FloatingMenuState = {
  isBold: boolean
  isItalic: boolean
}

type ToggleMenuProp = {
  pressed: boolean
  icon: ReactNode
  onClick: () => void
}

const menuIcons1: Partial<Record<TextFormatType, ReactNode>> = {
  bold: <Bold className='size-4' />,
  italic: <Italic className='size-4' />
}

function TogleMenu({ pressed, icon, onClick }: ToggleMenuProp) {
  return (
    <Toggle
      aria-label='Format text as bold'
      pressed={pressed}
      className='rounded-none data-[state=checked]:!bg-primary '
      onClick={onClick}
    >
      {icon}
    </Toggle>
  )
}

export function EditorFloatingMenu({ editor }: FloatingMenuComponentProps) {
  const [state, setState] = useState<FloatingMenuState>({
    isBold: false,
    isItalic: false
  })

  useEffect(() => {
    const unregisterListener = editor.registerUpdateListener(({ editorState }) => {
      editorState.read(() => {
        const selection = $getSelection()
        if (!$isRangeSelection(selection)) return

        setState({
          isBold: selection.hasFormat('bold'),
          isItalic: selection.hasFormat('italic')
        })
      })
    })
    return unregisterListener
  }, [editor])

  const onPressed = useCallback(
    (action: TextFormatType) => {
      editor.dispatchCommand(FORMAT_TEXT_COMMAND, action)
    },
    [editor]
  )

  return (
    <div className='flex items-center justify-between bg-sidebar rounded-2xl overflow-hidden'>
      <ToggleGroup size='sm' type='multiple'>
        <Toggle
          aria-label='Format text as bold'
          pressed={state.isBold}
          className='rounded-none data-[state=checked]:!bg-primary '
          onClick={() => {
            editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'bold')
          }}
        >
          <Bold className='size-4' />
        </Toggle>
        <Toggle
          aria-label='Format text as italic'
          pressed={state.isItalic}
          className='rounded-none'
          onClick={() => {
            editor.dispatchCommand(FORMAT_TEXT_COMMAND, 'italic')
          }}
        >
          <Italic className='size-4' />
        </Toggle>
      </ToggleGroup>
    </div>
  )
}
