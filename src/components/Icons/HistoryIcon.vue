<script setup lang="ts">
defineProps({
  size: { type: Number, default: 16 },
  color: { type: String, default: 'currentColor' },
  strokeWidth: { type: Number, default: 2 },
});
</script>

<template>
  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <circle cx="12" cy="12" r="10" />
    <polyline points="12 6 12 12 16 14" />
  </svg>
</template>