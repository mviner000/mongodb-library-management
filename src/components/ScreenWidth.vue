<template>
    <div class="p-4 text-center">
      <p class="text-lg font-semibold">
        Your screen width size is: <span class="text-blue-600">{{ screenWidth }}px</span>
      </p>
    </div>
  </template>
  
  <script lang="ts" setup>
  import { ref, onMounted, onBeforeUnmount } from 'vue'
  
  const screenWidth = ref(window.innerWidth)
  
  function updateWidth() {
    screenWidth.value = window.innerWidth
  }
  
  onMounted(() => {
    window.addEventListener('resize', updateWidth)
  })
  
  onBeforeUnmount(() => {
    window.removeEventListener('resize', updateWidth)
  })
  </script>
  