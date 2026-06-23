import axios from 'axios'

/** Pretty-print Python via the backend (Black → autopep8). Returns the original
 *  code on any failure so formatting never loses the user's work. */
export const formatCode = (code: string): Promise<string> =>
  axios
    .post<{ code: string; changed: boolean }>('/api/format', { code })
    .then((r) => r.data.code)
    .catch(() => code)

let registered = false

/** Register Python formatting providers once. Black formats whole files, so the
 *  range provider (used by format-on-paste) reformats the entire model too. */
export function registerPythonFormatter(monaco: any) {
  if (registered) return
  registered = true

  const formatWhole = async (model: any) => {
    const src = model.getValue()
    const formatted = await formatCode(src)
    if (formatted === src) return []
    return [{ range: model.getFullModelRange(), text: formatted }]
  }

  monaco.languages.registerDocumentFormattingEditProvider('python', {
    provideDocumentFormattingEdits: (model: any) => formatWhole(model),
  })
  monaco.languages.registerDocumentRangeFormattingEditProvider('python', {
    provideDocumentRangeFormattingEdits: (model: any) => formatWhole(model),
  })
}
