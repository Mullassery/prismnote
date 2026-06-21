import { useState } from 'react'
import { GitBranch, X, RefreshCw, GitCommit, ArrowUp, ArrowDown, GitFork, FolderGit2 } from 'lucide-react'
import { gitStatus, gitInit, gitClone, gitCommit, gitPush, gitPull, type GitStatus } from '../api/git'

// GitHub/Git integration — multiple ways: init, clone, stage+commit, push, pull,
// status. Operates on a directory the user points at (their notebook folder).
export default function GitPanel({ onClose }: { onClose: () => void }) {
  const [dir, setDir] = useState('')
  const [status, setStatus] = useState<GitStatus | null>(null)
  const [message, setMessage] = useState('')
  const [cloneUrl, setCloneUrl] = useState('')
  const [cloneDir, setCloneDir] = useState('')
  const [log, setLog] = useState('')
  const [busy, setBusy] = useState(false)

  const run = async (fn: () => Promise<{ ok?: boolean; output?: string } | GitStatus>) => {
    setBusy(true)
    try {
      const r: any = await fn()
      if (r.output !== undefined) setLog(r.output || (r.ok ? '✓ done' : 'failed'))
      if (dir) setStatus(await gitStatus(dir))
    } catch (e: any) {
      setLog(e?.message || 'error')
    } finally {
      setBusy(false)
    }
  }

  const refresh = async () => {
    if (!dir) return
    setBusy(true)
    try {
      setStatus(await gitStatus(dir))
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="absolute inset-0 z-30 pn-app flex flex-col">
      <div className="h-10 flex items-center justify-between px-4 border-b pn-bd">
        <span className="flex items-center gap-2 text-sm font-semibold pn-text">
          <GitBranch size={16} className="text-violet-400" /> Source Control
        </span>
        <button onClick={onClose} className="p-1 rounded pn-hover pn-muted"><X size={16} /></button>
      </div>

      <div className="flex-1 overflow-y-auto p-4 space-y-4 max-w-3xl">
        {/* Repo directory */}
        <div>
          <div className="text-[12px] pn-muted mb-1">Repository folder (absolute path)</div>
          <div className="flex gap-2">
            <input value={dir} onChange={(e) => setDir(e.target.value)} placeholder="/Users/you/my-notebooks"
              className="flex-1 px-2 py-1.5 rounded bg-white/5 border pn-bd pn-text text-[13px] outline-none focus:border-violet-500" />
            <button onClick={refresh} disabled={!dir || busy} className="flex items-center gap-1 px-3 rounded bg-white/5 hover:bg-white/10 pn-text text-[13px] disabled:opacity-40">
              <RefreshCw size={13} className={busy ? 'animate-spin' : ''} /> Status
            </button>
          </div>
        </div>

        {/* Status */}
        {status && (
          <div className="rounded-lg border pn-bd pn-solid-bg p-3 text-[13px]">
            {status.is_repo ? (
              <>
                <div className="flex items-center gap-2 pn-text"><GitBranch size={13} className="text-violet-400" /> {status.branch || 'detached'}</div>
                <pre className="mt-2 font-mono text-[12px] pn-muted whitespace-pre-wrap">{status.status || 'working tree clean'}</pre>
              </>
            ) : (
              <div className="flex items-center justify-between">
                <span className="pn-muted">Not a git repository.</span>
                <button onClick={() => run(() => gitInit(dir))} disabled={!dir || busy} className="flex items-center gap-1 px-2 py-1 rounded bg-violet-500/20 text-violet-200 text-[12px]"><FolderGit2 size={13} /> git init</button>
              </div>
            )}
          </div>
        )}

        {/* Commit / push / pull */}
        <div className="rounded-lg border pn-bd p-3 space-y-2">
          <div className="text-[12px] pn-muted">Commit &amp; sync</div>
          <input value={message} onChange={(e) => setMessage(e.target.value)} placeholder="Commit message"
            className="w-full px-2 py-1.5 rounded bg-white/5 border pn-bd pn-text text-[13px] outline-none focus:border-violet-500" />
          <div className="flex gap-2">
            <button onClick={() => run(() => gitCommit(dir, message || 'Update from PrismNote'))} disabled={!dir || busy}
              className="flex items-center gap-1 px-3 py-1.5 rounded prism-bg text-white text-[13px] disabled:opacity-40"><GitCommit size={14} /> Commit</button>
            <button onClick={() => run(() => gitPush(dir))} disabled={!dir || busy}
              className="flex items-center gap-1 px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 pn-text text-[13px] disabled:opacity-40"><ArrowUp size={14} /> Push</button>
            <button onClick={() => run(() => gitPull(dir))} disabled={!dir || busy}
              className="flex items-center gap-1 px-3 py-1.5 rounded bg-white/5 hover:bg-white/10 pn-text text-[13px] disabled:opacity-40"><ArrowDown size={14} /> Pull</button>
          </div>
        </div>

        {/* Clone */}
        <div className="rounded-lg border pn-bd p-3 space-y-2">
          <div className="text-[12px] pn-muted">Clone a repository</div>
          <input value={cloneUrl} onChange={(e) => setCloneUrl(e.target.value)} placeholder="https://github.com/user/repo.git"
            className="w-full px-2 py-1.5 rounded bg-white/5 border pn-bd pn-text text-[13px] outline-none focus:border-violet-500" />
          <input value={cloneDir} onChange={(e) => setCloneDir(e.target.value)} placeholder="destination folder (absolute path)"
            className="w-full px-2 py-1.5 rounded bg-white/5 border pn-bd pn-text text-[13px] outline-none focus:border-violet-500" />
          <button onClick={() => run(() => gitClone(cloneUrl, cloneDir))} disabled={!cloneUrl || !cloneDir || busy}
            className="flex items-center gap-1 px-3 py-1.5 rounded prism-bg text-white text-[13px] disabled:opacity-40"><GitFork size={14} /> Clone</button>
        </div>

        {log && <pre className="font-mono text-[12px] pn-muted whitespace-pre-wrap border-t pn-bd pt-2">{log}</pre>}
      </div>
    </div>
  )
}
