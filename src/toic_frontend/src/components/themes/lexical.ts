const lexicalTheme = {
  paragraph: 'mb-4 text-base leading-relaxed text-gray-800',
  heading: {
    h1: 'text-3xl font-bold mb-4',
    h2: 'text-2xl font-semibold mb-3',
    h3: 'text-xl font-semibold mb-2'
  },
  text: {
    bold: 'font-bold',
    italic: 'italic',
    underline: 'underline',
    strikethrough: 'line-through'
  },
  list: {
    ul: 'list-disc list-inside',
    ol: 'list-decimal list-inside',
    listitem: 'mb-2 ml-4'
  },
  quote: 'border-l-4 border-gray-300 pl-4 italic text-gray-600 mb-4',
  code: 'bg-gray-100 font-mono px-1 py-0.5 rounded',
  codeHighlight: {
    atrule: 'text-red-500',
    attr: 'text-green-500',
    boolean: 'text-blue-500'
  }
}

export default lexicalTheme
