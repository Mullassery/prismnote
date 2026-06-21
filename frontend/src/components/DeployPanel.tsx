import { useEffect, useState } from 'react'
import axios from 'axios'
import { Rocket, X, Copy, Download, Loader2 } from 'lucide-react'

// Cloud deployment made easy: shows ready-to-use Dockerfile, docker-compose,
// Kubernetes manifest, and fly.toml with copy/download + the one-line command.
export default function DeployPanel({ onClose }: { onClose: () => void }) {
  const [artifacts, setArtifacts] = useState<Record<string, any> | null>(null)
  const [tab, setTab] = useState('Dockerfile')
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    axios.get('/api/deploy/artifacts').then((r) => setArtifacts(r.data)).finally(() => setLoading(false))
  }, [])

  const files = artifacts ? Object.keys(artifacts).filter((k) => k !== 'commands') : []
  const content = artifacts?.[tab] ?? ''
  const commands: Record<string, string> = artifacts?.commands ?? {}
  const cmdFor =
    tab === 'Dockerfile' || tab === 'docker-compose.yml' ? commands.docker
    : tab === 'k8s.yaml' ? commands.kubernetes
    : tab === 'fly.toml' ? commands.fly
    : ''

  const download = () => {
    const blob = new Blob([content], { type: 'text/plain' })
    const a = document.createElement('a')
    a.href = URL.createObjectURL(blob)
    a.download = tab
    a.click()
    URL.revokeObjectURL(a.href)
  }

  return (
    <div className="absolute inset-0 z-30 pn-app flex flex-col">
      <div className="h-10 flex items-center justify-between px-4 border-b pn-bd">
        <span className="flex items-center gap-2 text-sm font-semibold pn-text">
          <Rocket size={16} className="text-violet-400" /> Deploy to Cloud
        </span>
        <button onClick={onClose} className="p-1 rounded pn-hover pn-muted"><X size={16} /></button>
      </div>

      {loading ? (
        <div className="p-4 pn-faint flex items-center gap-2"><Loader2 size={14} className="animate-spin" /> Generating artifacts…</div>
      ) : (
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex items-center gap-1 px-3 py-1.5 border-b pn-bd">
            {files.map((f) => (
              <button key={f} onClick={() => setTab(f)}
                className={`px-2.5 py-1 rounded text-[12px] ${tab === f ? 'bg-violet-500/30 text-violet-200' : 'pn-muted hover:pn-text'}`}>
                {f}
              </button>
            ))}
            <div className="flex-1" />
            <button onClick={() => navigator.clipboard.writeText(content)} className="flex items-center gap-1 px-2 py-1 rounded bg-white/5 hover:bg-white/10 pn-text text-[12px]"><Copy size={12} /> Copy</button>
            <button onClick={download} className="flex items-center gap-1 px-2 py-1 rounded bg-white/5 hover:bg-white/10 pn-text text-[12px]"><Download size={12} /> Download</button>
          </div>
          {cmdFor && (
            <div className="px-3 py-2 text-[12px] border-b pn-bd bg-violet-500/5">
              Deploy: <code className="pn-text">{cmdFor}</code>
            </div>
          )}
          <pre className="flex-1 overflow-auto p-3 text-[12px] font-mono pn-muted whitespace-pre">{content}</pre>
        </div>
      )}
    </div>
  )
}
