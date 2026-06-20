interface OutputProps {
  output: any
}

export default function Output({ output }: OutputProps) {
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
      return (
        <div className="bg-red-900 bg-opacity-20 border border-red-700 p-3 rounded text-red-300 text-sm">
          <pre className="font-mono overflow-x-auto">
            {Array.isArray(output.traceback) ? output.traceback.join('\n') : output.text}
          </pre>
        </div>
      )
    default:
      return null
  }
}
