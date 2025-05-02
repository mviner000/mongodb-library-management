<!-- src/components/Icons/ShieldIcon.vue -->
<script setup lang="ts">
defineProps({
  size: { type: Number, default: 14 },
  color: { type: String, default: 'currentColor' },
  strokeWidth: { type: Number, default: 2 },
});
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 24 24" fill="none" :stroke="color" :stroke-width="strokeWidth" stroke-linecap="round" stroke-linejoin="round">
    <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"></path>
  </svg>
</template>