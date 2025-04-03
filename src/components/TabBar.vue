<!-- src/components/TabBar.vue -->
<script setup lang="ts">
defineProps<{
  tabs: Array<{
    id: string
    title: string
    type: string
  }>
  activeTabId: string
}>()

const emit = defineEmits(['closeTab', 'tabClick'])
</script>

<template>
  <div class="flex bg-[#272822] w-full h-9 border-b border-gray-800 overflow-x-auto">
    <div 
      v-for="tab in tabs" 
      :key="tab.id" 
      @click="emit('tabClick', tab.id)"
      class="group flex items-center min-w-[140px] max-w-[200px] px-3 py-1 border-r border-gray-800 text-sm text-gray-400 cursor-pointer"
      :class="{ 'bg-[#3c3c3c] text-gray-300': activeTabId === tab.id }"
    >
      <span class="flex-1 truncate">{{ tab.title }}</span>
      <button 
        @click.stop="emit('closeTab', tab.id)" 
        class="w-4 h-4 flex items-center justify-center rounded-sm hover:bg-gray-700"
        >
        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
        </button>
    </div>
    <div class="flex-1 bg-[#272822]"></div>
  </div>
</template>