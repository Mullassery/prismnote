import { useEffect, useState } from 'react'
import './App.css'
import './styles/animations.css'
import './styles/components.css'
import Notebook from './components/Notebook'
import Sidebar from './components/Sidebar'
import UnifiedSearch from './components/UnifiedSearch'
import { useNotebookStore } from './hooks/useNotebook'

function App() {
  const [theme, setTheme] = useState<'light' | 'dark'>('dark')
  const { currentNotebookId } = useNotebookStore()

  useEffect(() => {
    const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
    setTheme(prefersDark ? 'dark' : 'light')
    document.documentElement.classList.toggle('dark', prefersDark)
  }, [])

  const toggleTheme = () => {
    const newTheme = theme === 'light' ? 'dark' : 'light'
    setTheme(newTheme)
    document.documentElement.classList.toggle('dark', newTheme === 'dark')
    localStorage.setItem('theme', newTheme)
  }

  return (
    <div className={`${theme} min-h-screen flex flex-col bg-white dark:bg-gray-900 text-gray-900 dark:text-white`}>
      <Sidebar onToggleTheme={toggleTheme} currentTheme={theme} />
      <main className="flex-1 overflow-hidden flex">
        {currentNotebookId ? (
          <Notebook />
        ) : (
          <div className="flex-1 flex items-center justify-center">
            <div className="text-center">
              <h1 className="text-4xl font-bold mb-4 text-gray-900 dark:text-white">Welcome to PrismNote</h1>
              <p className="text-lg text-gray-600 dark:text-gray-400">Create a new notebook to get started</p>
            </div>
          </div>
        )}
      </main>
      <UnifiedSearch />
    </div>
  )
}

export default App
