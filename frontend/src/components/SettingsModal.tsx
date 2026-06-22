import { useEffect, useState } from 'react'
import { X, Palette, Type, Columns3, Bot, Check } from 'lucide-react'

interface Props {
  onClose: () => void
  theme: 'light' | 'dark'
  setTheme: (t: 'light' | 'dark') => void
  panels: { files: boolean; terminal: boolean; ai: boolean }
  togglePanel: (p: 'files' | 'terminal' | 'ai') => void
}

function Row({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4 py-2.5">
      <div className="min-w-0">
        <div className="text-[13px] pn-text">{label}</div>
        {hint && <div className="text-[12px] pn-faint">{hint}</div>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  )
}

function Section({ icon, title, children }: { icon: React.ReactNode; title: string; children: React.ReactNode }) {
  return (
    <div className="py-3 border-b pn-bd last:border-0">
      <div className="flex items-center gap-2 text-[11px] font-semibold uppercase tracking-wider pn-muted mb-1">
        {icon} {title}
      </div>
      {children}
    </div>
  )
}

function Toggle({ on, onClick }: { on: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className={`w-9 h-5 rounded-full transition-colors relative ${on ? 'prism-bg' : 'bg-[var(--pn-hover)]'}`}
    >
      <span className={`absolute top-0.5 w-4 h-4 rounded-full bg-white shadow transition-all ${on ? 'left-[18px]' : 'left-0.5'}`} />
    </button>
  )
}

export default function SettingsModal({ onClose, theme, setTheme, panels, togglePanel }: Props) {
  const [fontSize, setFontSize] = useState<number>(() => Math.max(20, parseInt(localStorage.getItem('pn-code-size') || '20', 10)))
  const [endpoint, setEndpoint] = useState<string>(() => localStorage.getItem('pn-ollama') || 'http://localhost:11434')

  useEffect(() => {
    document.documentElement.style.setProperty('--pn-code-size', `${fontSize}px`)
    localStorage.setItem('pn-code-size', String(fontSize))
  }, [fontSize])

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && onClose()
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [onClose])

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center" onMouseDown={onClose}>
      <div className="absolute inset-0 bg-black/40 backdrop-blur-[2px]" />
      <div
        className="relative w-[560px] max-w-[92vw] max-h-[80vh] overflow-y-auto pn-solid-bg border pn-bd rounded-2xl shadow-2xl shadow-black/50"
        onMouseDown={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between px-5 h-12 border-b pn-bd sticky top-0 pn-solid-bg">
          <h2 className="text-[15px] font-semibold pn-text">Settings</h2>
          <button onClick={onClose} className="p-1.5 rounded-lg pn-hover pn-muted hover:pn-text"><X size={16} /></button>
        </div>

        <div className="px-5">
          <Section icon={<Palette size={13} />} title="Appearance">
            <Row label="Theme" hint="Switch between the warm dark and light palette">
              <div className="flex rounded-lg bg-[var(--pn-hover)] p-0.5 text-[12px]">
                {(['dark', 'light'] as const).map((t) => (
                  <button
                    key={t}
                    onClick={() => setTheme(t)}
                    className={`px-3 py-1 rounded-md capitalize ${theme === t ? 'prism-bg text-white' : 'pn-muted'}`}
                  >
                    {t}
                  </button>
                ))}
              </div>
            </Row>
          </Section>

          <Section icon={<Type size={13} />} title="Editor">
            <Row label="Code font size" hint={`${fontSize}px · terminal, output, code`}>
              <input
                type="range"
                min={20}
                max={32}
                value={fontSize}
                onChange={(e) => setFontSize(parseInt(e.target.value, 10))}
                className="w-40 accent-blue-500"
              />
            </Row>
            <Row label="Preview">
              <code className="pn-code pn-code-size px-2 py-1 rounded pn-solid-bg border pn-bd">df.head(10)</code>
            </Row>
          </Section>

          <Section icon={<Columns3 size={13} />} title="Layout">
            <Row label="File Explorer"><Toggle on={panels.files} onClick={() => togglePanel('files')} /></Row>
            <Row label="Terminal panel"><Toggle on={panels.terminal} onClick={() => togglePanel('terminal')} /></Row>
            <Row label="AI Assistant"><Toggle on={panels.ai} onClick={() => togglePanel('ai')} /></Row>
          </Section>

          <Section icon={<Bot size={13} />} title="AI (Ollama)">
            <Row label="Ollama endpoint" hint="Local model server URL">
              <input
                value={endpoint}
                onChange={(e) => {
                  setEndpoint(e.target.value)
                  localStorage.setItem('pn-ollama', e.target.value)
                }}
                className="w-52 text-[12px] px-2 py-1 rounded-lg pn-solid-bg border pn-bd pn-text outline-none focus:border-blue-500/60"
              />
            </Row>
            <div className="flex items-center gap-1.5 text-[12px] pn-faint pb-1">
              <Check size={13} className="text-emerald-400" /> Models are auto-discovered from this endpoint.
            </div>
          </Section>
        </div>
      </div>
    </div>
  )
}
