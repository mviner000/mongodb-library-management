<!-- src/components/Icons/MinusIcon.vue -->
<script setup lang="ts">
defineProps({
  size: { type: Number, default: 16 },
  color: { type: String, default: 'currentColor' },
  strokeWidth: { type: Number, default: 2 },
});
</script>

<template>
  <svg :width="size" :height="size" viewBox="0 0 24 24" fill="none" :stroke="color" :stroke-width="strokeWidth">
    <path d="M5 12h14" />
  </svg>
</template>
