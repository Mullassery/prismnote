import { useEffect, useState } from 'react'
import './App.css'
import Notebook from './components/Notebook'
import Sidebar from './components/Sidebar'
import { useNotebookStore } from './hooks/useNotebook'

function App() {
  const [theme, setTheme] = useState<'light' | 'dark'>('dark')
  const { currentNotebookId } = useNotebookStore()

  useEffect(() => {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    setTheme(prefersDark ? 'dark' : 'light')
  }, [])

  return (
    <div className={`${theme} h-screen flex bg-slate-950 text-white`}>
      <Sidebar />
      <main className="flex-1 overflow-hidden">
        {currentNotebookId ? (
          <Notebook />
        ) : (
          <div className="flex items-center justify-center h-full text-gray-400">
            <div className="text-center">
              <h1 className="text-3xl font-bold mb-4">Welcome to PrismNote</h1>
              <p>Create a new notebook to get started</p>
            </div>
          </div>
        )}
      </main>
    </div>
  )
}

export default App
