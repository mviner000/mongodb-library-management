<script setup lang="ts">
defineProps({
  size: { type: Number, default: 16 },
  color: { type: String, default: 'currentColor' },
  strokeWidth: { type: Number, default: 2 },
});
</script>

<template>
  <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" />
    <path d="M4 8h16" />
  </svg>
</template>