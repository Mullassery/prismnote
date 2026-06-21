import { useState } from 'react'

/** Per-panel font size, persisted in localStorage under `key`. */
export function useFontSize(key: string, def = 13, min = 9, max = 28) {
  const [size, setSize] = useState(() => {
    const v = parseInt(localStorage.getItem(key) || '', 10)
    return Number.isFinite(v) && v > 0 ? v : def
  })
  const apply = (n: number) => {
    const clamped = Math.max(min, Math.min(max, n))
    setSize(clamped)
    localStorage.setItem(key, String(clamped))
  }
  return { size, inc: () => apply(size + 1), dec: () => apply(size - 1) }
}
