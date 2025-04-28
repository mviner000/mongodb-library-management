<script setup lang="ts">
  import { computed, PropType } from 'vue' // [cite: 1]
  import {
    NavigationMenu,
    NavigationMenuContent,
    NavigationMenuItem,
    NavigationMenuList,
    NavigationMenuTrigger,
  } from './ui/navigation-menu' // [cite: 3]

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
    return !props.hiddenColumns.includes(header) // [cite: 106]
  }

  const handleToggle = (header: string) => {
    emit('toggle-visibility', header) // [cite: 106]
  }
</script>

<template>
  <NavigationMenu>
    <NavigationMenuList>
      <NavigationMenuItem>
        <NavigationMenuTrigger size="sm">Visibility</NavigationMenuTrigger>
        <NavigationMenuContent>
          <div class="p-2 w-48">
            <div
              v-for="header in tableHeaders"
              :key="header"
              class="flex items-center space-x-2 py-1 px-2 rounded hover:bg-green-100 transition-colors duration-150 group"
            >
              <input
                type="checkbox"
                :id="`checkbox-${header}-${Math.random()}`"
                :checked="isVisible(header)"
                @change="handleToggle(header)"
                class="cursor-pointer accent-green-600"
              />
              <label
                :for="`checkbox-${header}-${Math.random()}`"
                class="flex-1 cursor-pointer text-sm group-hover:text-green-800"
              >
                {{ shortNames[header] || header }}
              </label>
            </div>
          </div>
        </NavigationMenuContent>
      </NavigationMenuItem>
    </NavigationMenuList>
  </NavigationMenu>
</template>

<style scoped>
  /* Add any component-specific styles here if needed, but the goal is to use existing UI styles */
</style>
