// Parse a Python traceback into something humans (and the editor) can use:
// the error type/message, the offending line/column within the cell, and a
// plain-language explanation + hint.

export interface ParsedError {
  ename: string
  evalue: string
  line?: number // 1-based line within the cell
  col?: number //  1-based column (when known, e.g. SyntaxError)
  friendly: string // natural-language explanation + hint
  traceback: string
}

/** Turn an error `Output` (or raw traceback text) into a ParsedError. */
export function parseTraceback(text: string): ParsedError {
  const tb = (text || '').replace(/\r/g, '')
  const lines = tb.split('\n').filter((l) => l.length > 0)

  // Last "Type: message" line is the actual error.
  let ename = 'Error'
  let evalue = ''
  for (let i = lines.length - 1; i >= 0; i--) {
    const m = lines[i].match(/^([A-Za-z_][\w.]*(?:Error|Exception|Warning|Interrupt|Exit)):?\s?(.*)$/)
    if (m) {
      ename = m[1]
      evalue = m[2] ?? ''
      break
    }
  }

  // Deepest user-cell frame gives the line within the cell.
  let line: number | undefined
  const frameRe = /File "(?:<cell>|<string>|<unknown>)", line (\d+)/g
  let fm: RegExpExecArray | null
  while ((fm = frameRe.exec(tb))) line = parseInt(fm[1], 10)

  // SyntaxError carries a caret line ("    ^") pointing at the column.
  let col: number | undefined
  const caretIdx = tb.split('\n').findIndex((l) => /^\s*\^+\s*$/.test(l))
  if (caretIdx >= 0) {
    col = tb.split('\n')[caretIdx].indexOf('^') + 1
  }

  return { ename, evalue, line, col, friendly: friendlyError(ename, evalue), traceback: tb }
}

/** Map a Python error type + message to a friendly explanation and a fix hint. */
export function friendlyError(ename: string, evalue: string): string {
  const name = pick(evalue, /'([^']+)'/) || pick(evalue, /name (\w+)/)
  switch (ename) {
    case 'NameError':
      return `Python doesn't recognise the name ${code(name)}. It's likely undefined, misspelled, or defined in a cell you haven't run yet — run the earlier cell first.`
    case 'ModuleNotFoundError':
    case 'ImportError':
      return `The module ${code(name)} isn't available in this kernel. Install it from a cell with ${code('!pip install ' + (name ?? 'package'))}, then re-run.`
    case 'KeyError':
      return `The key ${code(name)} doesn't exist in that dict/DataFrame. Check the available keys (e.g. ${code('df.columns')} or ${code('d.keys()')}).`
    case 'IndexError':
      return `You indexed past the end of a list/array. The index is out of range for its current length.`
    case 'AttributeError':
      return `That object has no attribute or method ${code(name)}. Check the spelling, or that the variable is the type you expect.`
    case 'TypeError':
      return `An operation received a value of the wrong type (${evalue}). Check the types of the values involved — a common cause is mixing str and int, or calling something that isn't a function.`
    case 'ValueError':
      return `The value is the right type but unacceptable here (${evalue}). Often a bad conversion or a shape/length mismatch.`
    case 'ZeroDivisionError':
      return `You divided by zero. Guard the denominator (e.g. ${code('x / y if y else 0')}).`
    case 'FileNotFoundError':
      return `No file exists at that path. Check the path and the working directory (${code('import os; os.getcwd()')}).`
    case 'IndentationError':
      return `The indentation is off on this line. Python uses consistent spaces — make sure nested blocks line up.`
    case 'SyntaxError':
      return `Python couldn't parse this line. Look for a missing colon, closing bracket/quote, or a stray character right around the highlighted spot.`
    case 'KeyboardInterrupt':
      return `Execution was interrupted (Stop). The cell didn't finish.`
    default:
      return evalue ? `${ename}: ${evalue}` : `${ename} occurred while running this cell.`
  }
}

function pick(s: string, re: RegExp): string | undefined {
  const m = s.match(re)
  return m?.[1]
}
function code(s?: string) {
  return s ? `\`${s}\`` : 'that name'
}
