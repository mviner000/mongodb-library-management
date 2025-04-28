<script setup lang="ts">
  import { computed, PropType } from 'vue'
  // Import DropdownMenu components and Button
  import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuTrigger,
    // DropdownMenuItem, DropdownMenuCheckboxItem, DropdownMenuLabel, DropdownMenuSeparator could also be used if needed for more complex menus
  } from '../ui/dropdown-menu' //
  import { Button } from '../ui/button' //

  // Define the expected structure for the schema, focusing on what's needed
  interface SimpleSchema {
    ui?: {
      short_names?: Record<string, string>
    }
  }

  const props = defineProps({
    tableHeaders: {
      type: Array as PropType<string[]>,
      required: true,
    },
    hiddenColumns: {
      type: Array as PropType<string[]>,
      required: true,
    },
    // Pass only the necessary part of the schema
    collectionSchema: {
      type: Object as PropType<SimpleSchema>,
      required: true,
    },
  })

  const emit = defineEmits<{
    (e: 'toggle-visibility', header: string): void
  }>()

  // Helper computed property to safely access short names
  const shortNames = computed(() => props.collectionSchema?.ui?.short_names || {})

  const isVisible = (header: string) => {
    // No change needed here
    return !props.hiddenColumns.includes(header)
  }

  const handleToggle = (header: string) => {
    // No change needed here
    emit('toggle-visibility', header)
  }

  // Helper function to generate a stable ID based on the header
  const generateCheckboxId = (header: string) => {
    // Using header ensures stability compared to Math.random()
    // Replace potential problematic characters for ID if necessary, though unlikely for headers
    const safeHeader = header.replace(/[^a-zA-Z0-9-_]/g, '_')
    return `visibility-checkbox-${safeHeader}`
  }
</script>

<template>
  <DropdownMenu>
    <DropdownMenuTrigger as-child>
      <Button
        variant="outline"
        size="sm"
        >Visibility</Button
      >
    </DropdownMenuTrigger>
    <DropdownMenuContent class="w-48 p-2">
      <div
        v-for="header in tableHeaders"
        :key="header"
        class="flex items-center space-x-2 py-1 px-2 rounded hover:bg-green-100 transition-colors duration-150 group cursor-pointer"
        @click.stop="handleToggle(header)"
      >
        <input
          type="checkbox"
          :id="generateCheckboxId(header)"
          :checked="isVisible(header)"
          @change.stop="handleToggle(header)"
          @click.stop
          class="cursor-pointer accent-green-600 pointer-events-none"
        />
        <label
          :for="generateCheckboxId(header)"
          class="flex-1 cursor-pointer text-sm group-hover:text-green-800 select-none"
          @click.prevent
        >
          {{ shortNames[header] || header }}
        </label>
      </div>
    </DropdownMenuContent>
  </DropdownMenu>
</template>

<style scoped>
  /* Add any component-specific styles here if needed */
  /* Ensure dropdown content allows interaction without closing immediately if needed */
</style>
