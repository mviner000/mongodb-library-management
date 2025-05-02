<!-- src/components/TabBar.vue -->
<!-- src/components/TabBar.vue -->
<template>
  <div class="flex border-b">
    <div class="flex-1 flex overflow-x-auto">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        class="flex items-center px-4 py-2 border-r cursor-pointer whitespace-nowrap"
        :class="{ 'bg-blue-50': tab.id === activeTabId }"
        @click="$emit('tab-click', tab.id)"
      >
        <span class="text-xs text-gray-500 mr-1">[{{ tab.id }}]</span> {{ tab.title }}
        <button
          v-if="tabs.length > 1"
          class="ml-2 text-gray-500 hover:text-gray-700"
          @click.stop="$emit('close-tab', tab.id)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M18 6L6 18M6 6l12 12"></path>
          </svg>
        </button>
      </div>
    </div>
    <div v-if="showAddButton" class="flex items-center px-2 border-l">
      <button
        class="p-1 text-gray-500 hover:text-gray-700"
        @click="$emit('add-tab')"
        title="New Tab (Ctrl+T)"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M12 5v14M5 12h14"></path>
        </svg>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps({
  tabs: {
    type: Array as () => Array<{
      id: string;
      title: string;
      type: string;
      path?: string;
      reloadCount?: number;
    }>,
    required: true
  },
  activeTabId: {
    type: String,
    required: true
  },
  showAddButton: {
    type: Boolean,
    default: true
  }
})

defineEmits(['tab-click', 'close-tab', 'add-tab'])
</script>