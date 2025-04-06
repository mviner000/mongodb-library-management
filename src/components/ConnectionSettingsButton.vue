<!-- src/components/ConnectionSettingsButton.vue -->
<script setup lang="ts">
import SettingsIcon2 from './Icons/SettingsIcon2.vue'

const emit = defineEmits(['click'])

const handleClick = () => {
  emit('click')
}
</script>

<template>
  <button 
    @click="handleClick"
    class="fixed bottom-4 right-4 z-50 flex items-center px-3 py-2 text-sm font-medium text-white bg-[#8B5E3C] hover:bg-[#6F4E24] rounded-full shadow-lg transition-colors duration-200 ease-in-out"
  >
    <SettingsIcon2 class="h-5 w-5 mr-1"/>
    Edit Connection Settings
  </button>
</template>