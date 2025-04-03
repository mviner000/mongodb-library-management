<!-- src/components/HelloWorldTab.vue -->
<template>
    <div class="h-full w-full flex items-center justify-center">
      <h1 class="text-4xl font-bold">{{ content }}</h1>
    </div>
  </template>
  
  <script setup lang="ts">
  defineProps<{
    content: string
  }>()
  </script>