<!-- src/components/Sidebar.vue -->
<script setup lang="ts">
import { inject, computed } from 'vue'
import { Button } from '@/components/ui/button'

defineProps<{
  activeTab: 'home' | 'settings'
}>()

const emit = defineEmits(['selectTab'])

// Get sidebar state from parent
const sidebarState = inject('sidebarState') as {
  isOpen: { value: boolean },
  toggle: () => void,
  close: () => void
}

const handleTabSelect = (tab: 'home' | 'settings') => {
  emit('selectTab', tab)
  sidebarState.close()
}

// Computed class for sidebar position
const sidebarClass = computed(() => {
  return sidebarState.isOpen.value 
    ? 'translate-x-0 opacity-100' 
    : '-translate-x-full md:-translate-x-full opacity-0 md:opacity-0'
})
</script>

<template>
  <div 
    class="fixed inset-0 bg-black/20 z-30"
    v-if="sidebarState.isOpen.value"
    @click="sidebarState.close()"
  ></div>
  
  <div 
    class="fixed z-40 h-[calc(100vh-3.5rem)] top-14 bg-white border-r w-64 transition-all duration-150 ease-in-out"
    :class="sidebarClass"
  >
    <div class="flex flex-col gap-1 p-2 mt-20">
      <Button
        variant="ghost"
        :class="activeTab === 'home' ? 'bg-muted' : 'hover:bg-muted'"
        @click="handleTabSelect('home')"
        class="justify-start gap-2"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path>
          <polyline points="9 22 9 12 15 12 15 22"></polyline>
        </svg>
        <span>Home</span>
      </Button>
      <Button
        variant="ghost"
        :class="activeTab === 'settings' ? 'bg-muted' : 'hover:bg-muted'"
        @click="handleTabSelect('settings')"
        class="justify-start gap-2"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="3"></circle>
          <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
        <span>Settings</span>
      </Button>
    </div>
  </div>
</template>