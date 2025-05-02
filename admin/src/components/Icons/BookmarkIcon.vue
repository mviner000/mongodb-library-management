<!-- src/components/Icons/BookmarkIcon.vue -->
<script setup lang="ts">
defineProps({
  size: { type: Number, default: 16 },
  color: { type: String, default: 'currentColor' },
  strokeWidth: { type: Number, default: 2 },
});
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 24 24" fill="none" :stroke="color" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round">
    <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
  </svg>
</template>
