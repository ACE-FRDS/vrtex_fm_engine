type XmlToken = {
  kind: 'tag' | 'text' | 'cdata' | 'comment' | 'declaration'
  value: string
}

function readUntil(xml: string, start: number, marker: string) {
  const end = xml.indexOf(marker, start)
  return end < 0 ? xml.length : end + marker.length
}

function readTagEnd(xml: string, start: number) {
  let quote = ''
  let subsetDepth = 0
  for (let index = start + 1; index < xml.length; index += 1) {
    const character = xml[index] ?? ''
    if (quote) {
      if (character === quote) quote = ''
      continue
    }
    if (character === '"' || character === "'") {
      quote = character
      continue
    }
    if (character === '[') subsetDepth += 1
    else if (character === ']') subsetDepth = Math.max(0, subsetDepth - 1)
    else if (character === '>' && subsetDepth === 0) return index + 1
  }
  return xml.length
}

function tokenizeXml(xml: string) {
  const tokens: XmlToken[] = []
  let index = 0
  while (index < xml.length) {
    if (xml[index] !== '<') {
      const end = xml.indexOf('<', index)
      const next = end < 0 ? xml.length : end
      tokens.push({ kind: 'text', value: xml.slice(index, next) })
      index = next
      continue
    }

    if (xml.startsWith('<![CDATA[', index)) {
      const end = readUntil(xml, index + 9, ']]>')
      tokens.push({ kind: 'cdata', value: xml.slice(index, end) })
      index = end
      continue
    }
    if (xml.startsWith('<!--', index)) {
      const end = readUntil(xml, index + 4, '-->')
      tokens.push({ kind: 'comment', value: xml.slice(index, end) })
      index = end
      continue
    }
    if (xml.startsWith('<?', index)) {
      const end = readUntil(xml, index + 2, '?>')
      tokens.push({ kind: 'declaration', value: xml.slice(index, end) })
      index = end
      continue
    }

    const end = readTagEnd(xml, index)
    tokens.push({
      kind: xml.startsWith('<!', index) ? 'declaration' : 'tag',
      value: xml.slice(index, end),
    })
    index = end
  }
  return tokens
}

function tagName(tag: string) {
  return tag.match(/^<\/?\s*([^\s/>]+)/)?.[1] ?? ''
}

/**
 * Adds display-only line breaks and indentation without parsing and rebuilding
 * FileMaker XML. Attribute order, CDATA, calculations, and the original history
 * payload remain unchanged.
 */
export function formatXmlForDisplay(xml: string, indentation = '  ') {
  const source = xml.trim()
  if (!source || !source.includes('<')) return xml

  const parsed = new DOMParser().parseFromString(source, 'application/xml')
  if (parsed.querySelector('parsererror')) return xml

  const tokens = tokenizeXml(source)
  const lines: string[] = []
  let depth = 0
  for (let index = 0; index < tokens.length; index += 1) {
    const token = tokens[index]
    if (!token) continue
    if (token.kind === 'text' && !token.value.trim()) continue

    const value = token.kind === 'text' ? token.value.trim() : token.value
    const isClosingTag = token.kind === 'tag' && /^<\//.test(value)
    const isSelfClosingTag = token.kind === 'tag' && /\/\s*>$/.test(value)
    const isOpeningTag = token.kind === 'tag' && !isClosingTag && !isSelfClosingTag

    if (isOpeningTag) {
      const content = tokens[index + 1]
      const closing = tokens[index + 2]
      const closesInline = content
        && closing?.kind === 'tag'
        && /^<\//.test(closing.value)
        && tagName(value) === tagName(closing.value)
        && (content.kind === 'text' || content.kind === 'cdata')
      if (closesInline) {
        lines.push(`${indentation.repeat(depth)}${value}${content.value}${closing.value}`)
        index += 2
        continue
      }
      lines.push(`${indentation.repeat(depth)}${value}`)
      depth += 1
      continue
    }

    if (isClosingTag) depth = Math.max(0, depth - 1)
    lines.push(`${indentation.repeat(depth)}${value}`)
  }
  return lines.join('\n')
}
