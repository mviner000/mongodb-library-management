<!-- src/components/HorizontalScrollIndicator.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, computed } from 'vue';

const props = defineProps<{
  targetRef?: HTMLElement | null;
}>();

const scrollPercentage = ref(0);
const visibleStart = ref(0);
const visibleEnd = ref(0);
const totalWidth = ref(0);
const showBoom = ref(false);

let scrollElement: HTMLElement | Window = window;

// Action column start position is at totalWidth - 30
const actionColumnStart = computed(() => {
  return totalWidth.value - 30;
});

function updateScroll() {
  let scrollLeft: number;
  let scrollWidth: number;
  let clientWidth: number;

  if (scrollElement instanceof HTMLElement) {
    scrollLeft = scrollElement.scrollLeft;
    scrollWidth = scrollElement.scrollWidth;
    clientWidth = scrollElement.clientWidth;
  } else {
    scrollLeft = window.scrollX;
    scrollWidth = document.documentElement.scrollWidth;
    clientWidth = window.innerWidth;
  }

  const maxScroll = scrollWidth - clientWidth;
  scrollPercentage.value = maxScroll > 0 ? Math.round((scrollLeft / maxScroll) * 100) : 0;
  visibleStart.value = Math.round(scrollLeft);
  visibleEnd.value = Math.round(scrollLeft + clientWidth);
  totalWidth.value = Math.round(scrollWidth);
  
  // Check if Action column is visible
  showBoom.value = visibleEnd.value >= actionColumnStart.value;
}

function setupScrollListener() {
  if (scrollElement instanceof HTMLElement) {
    scrollElement.addEventListener('scroll', updateScroll, { passive: true });
  } else {
    window.addEventListener('scroll', updateScroll, { passive: true });
  }
  
  // Initial update without showing Boom
  let initialUpdate = true;
  if (initialUpdate) {
    // Just get the dimensions without setting showBoom
    if (scrollElement instanceof HTMLElement) {
      totalWidth.value = Math.round(scrollElement.scrollWidth);
      visibleStart.value = Math.round(scrollElement.scrollLeft);
      visibleEnd.value = Math.round(scrollElement.scrollLeft + scrollElement.clientWidth);
    } else {
      totalWidth.value = Math.round(document.documentElement.scrollWidth);
      visibleStart.value = Math.round(window.scrollX);
      visibleEnd.value = Math.round(window.scrollX + window.innerWidth);
    }
    
    // Force Boom to be hidden initially
    showBoom.value = false;
    initialUpdate = false;
  }
}

function removeScrollListener() {
  if (scrollElement instanceof HTMLElement) {
    scrollElement.removeEventListener('scroll', updateScroll);
  } else {
    window.removeEventListener('scroll', updateScroll);
  }
}

onMounted(() => {
  if (props.targetRef) {
    scrollElement = props.targetRef;
  }
  setupScrollListener();
  // Ensure Boom is hidden by default
  showBoom.value = false;
});

onUnmounted(() => {
  removeScrollListener();
});

watch(() => props.targetRef, (newTargetRef) => {
  removeScrollListener();
  scrollElement = newTargetRef || window;
  setupScrollListener();
  // Ensure Boom is hidden when target changes
  showBoom.value = false;
});
</script>

<template>
  <div class="fixed bottom-4 left-1/2 transform -translate-x-1/2 bg-blue-600 text-white px-3 py-1 rounded shadow">
    {{ visibleStart }}px - {{ visibleEnd }}px of {{ totalWidth }}px ({{ scrollPercentage }}%)
    <span v-if="showBoom" class="boom-text ml-2">Boom!</span>
  </div>
</template>

<style scoped>
div {
  transition: opacity 0.3s ease;
  pointer-events: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.boom-text {
  font-weight: bold;
  color: yellow;
  animation: pulse 0.5s infinite alternate;
}

@keyframes pulse {
  from {
    opacity: 0.7;
    transform: scale(1);
  }
  to {
    opacity: 1;
    transform: scale(1.1);
  }
}
</style>