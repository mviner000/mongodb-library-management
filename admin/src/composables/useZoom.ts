// src/components/useZoom.ts

import { ref, computed, onMounted, onUnmounted } from 'vue'

const MIN_ZOOM = 50
const MAX_ZOOM = 200
const ZOOM_STEP = 10

export function useZoom() {
  const zoomLevel = ref(100)

  const zoomStyle = computed(() => ({
    transform: `scale(${zoomLevel.value / 100})`,
    transformOrigin: 'top left',
    width: `${100 / (zoomLevel.value / 100)}%`,
    height: `${100 / (zoomLevel.value / 100)}%`
  }))

  function zoomIn() {
    zoomLevel.value = Math.min(MAX_ZOOM, zoomLevel.value + ZOOM_STEP)
    saveZoom()
  }

  function zoomOut() {
    zoomLevel.value = Math.max(MIN_ZOOM, zoomLevel.value - ZOOM_STEP)
    saveZoom()
  }

  function resetZoom() {
    zoomLevel.value = 100
    saveZoom()
  }

  function saveZoom() {
    localStorage.setItem('app-zoom-level', zoomLevel.value.toString())
  }

  function loadZoom() {
    const savedZoom = localStorage.getItem('app-zoom-level')
    if (savedZoom) zoomLevel.value = parseInt(savedZoom)
  }

  function handleKeyPress(e: KeyboardEvent) {
    if (e.ctrlKey) {
      if (e.key === '0') {
        e.preventDefault()
        resetZoom()
      } else if (e.key === '=' || e.key === '+') {
        e.preventDefault()
        zoomIn()
      } else if (e.key === '-') {
        e.preventDefault()
        zoomOut()
      }
    }
  }

  function handleWheel(e: WheelEvent) {
    if (e.ctrlKey) {
      e.preventDefault()
      e.deltaY < 0 ? zoomIn() : zoomOut()
    }
  }

  onMounted(() => {
    loadZoom()
    window.addEventListener('keydown', handleKeyPress)
    window.addEventListener('wheel', handleWheel, { passive: false })
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyPress)
    window.removeEventListener('wheel', handleWheel)
  })

  return {
    zoomLevel,
    zoomStyle,
    zoomIn,
    zoomOut,
    resetZoom
  }
}