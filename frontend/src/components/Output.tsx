import MDPreview from '@uiw/react-markdown-preview'
import { useState } from 'react'
import { AlertTriangle, ChevronRight } from 'lucide-react'
import { parseTraceback } from '../lib/pyerror'
import DataFrameView from './DataFrameView'

interface OutputProps {
  output: any
}

const DF_MIME = 'application/vnd.prismnote.df+json'

function ErrorOutput({ output }: { output: any }) {
  const [showTrace, setShowTrace] = useState(false)
  const raw = Array.isArray(output.traceback)
    ? output.traceback.join('\n')
    : Array.isArray(output.text)
    ? output.text.join('')
    : output.text || ''
  const parsed = parseTraceback(raw)
  return (
    <div className="bg-red-900/20 border border-red-700/60 rounded p-3 text-sm">
      <div className="flex items-start gap-2 text-red-300">
        <AlertTriangle size={16} className="mt-0.5 shrink-0" />
        <div className="min-w-0">
          <div className="font-semibold">
            {parsed.ename}
            {parsed.line ? <span className="font-normal text-red-300/80"> · line {parsed.line}{parsed.col ? `, col ${parsed.col}` : ''}</span> : null}
          </div>
          <div className="text-red-100/90 mt-0.5 leading-relaxed">{parsed.friendly}</div>
        </div>
      </div>
      <button
        onClick={() => setShowTrace((s) => !s)}
        className="mt-2 flex items-center gap-1 text-[12px] text-red-300/80 hover:text-red-200"
      >
        <ChevronRight size={13} className={showTrace ? 'rotate-90 transition-transform' : 'transition-transform'} />
        {showTrace ? 'Hide' : 'Show'} full traceback
      </button>
      {showTrace && (
        <pre className="mt-1 font-mono text-[12px] text-red-300/90 overflow-x-auto whitespace-pre">{raw}</pre>
      )}
    </div>
  )
}

export default function Output({ output }: OutputProps) {
  // %md magic and any text/markdown bundle render as formatted markdown.
  const md = output.data?.['text/markdown']
  if (md) {
    return (
      <div className="bg-slate-800 p-3 rounded">
        <MDPreview
          source={Array.isArray(md) ? md.join('') : md}
          style={{ backgroundColor: 'transparent', color: '#e5e7eb' }}
        />
      </div>
    )
  }

  // DataFrame results get a Table/Bar/Line switcher.
  const dfPayload = output.data?.[DF_MIME]
  if (dfPayload && (output.output_type === 'execute_result' || output.output_type === 'display_data')) {
    return <DataFrameView df={dfPayload} html={output.data?.['text/html']} />
  }

  switch (output.output_type) {
    case 'stream':
      return (
        <pre className="bg-slate-800 p-3 rounded text-sm text-gray-300 overflow-x-auto font-mono">
          {Array.isArray(output.text) ? output.text.join('') : output.text}
        </pre>
      )
    case 'execute_result':
      return (
        <div className="bg-slate-800 p-3 rounded text-sm text-gray-300">
          {output.data?.['text/plain'] && (
            <pre className="font-mono overflow-x-auto">
              {Array.isArray(output.data['text/plain'])
                ? output.data['text/plain'].join('')
                : output.data['text/plain']}
            </pre>
          )}
          {output.data?.['text/html'] && (
            <div
              className="viz-container"
              dangerouslySetInnerHTML={{ __html: output.data['text/html'] }}
            />
          )}
        </div>
      )
    case 'display_data':
      return (
        <div className="bg-slate-800 p-3 rounded">
          {output.data?.['image/png'] && (
            <img
              src={`data:image/png;base64,${output.data['image/png']}`}
              alt="output"
              className="viz-container max-w-full h-auto rounded"
              style={{ imageRendering: 'crisp-edges' }}
            />
          )}
          {output.data?.['text/html'] && (
            <div
              className="viz-container"
              dangerouslySetInnerHTML={{ __html: output.data['text/html'] }}
            />
          )}
          {output.data?.['application/json'] && (
            <pre className="font-mono text-xs overflow-x-auto text-gray-300">
              {JSON.stringify(output.data['application/json'], null, 2)}
            </pre>
          )}
        </div>
      )
    case 'error':
      return <ErrorOutput output={output} />
    default:
      return null
  }
}
