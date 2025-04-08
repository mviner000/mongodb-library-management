<script setup lang="ts">
// Import necessary functions from Vue 3 Composition API
import { ref, onMounted, onUnmounted, watch } from 'vue';

// --- Props ---
// Define component props using defineProps with TypeScript for type safety.
const props = defineProps<{
  // targetRef: An optional prop. If provided, it should be an HTMLElement.
  // The component will track the horizontal scroll percentage of this specific element.
  // If omitted or null, the component defaults to tracking the window's horizontal scroll.
  targetRef?: HTMLElement | null;
}>();

// --- State ---
// Reactive variable to store the calculated horizontal scroll percentage.
// 'ref' makes it reactive, so the template updates when its value changes.
const scrollPercentage = ref(0);

// Variable to hold the element whose scroll event we are listening to.
// It can be either the window object or a specific HTMLElement passed via props.
// Defaults to the global window object.
let scrollElement: HTMLElement | Window = window;

// --- Methods ---
/**
 * Calculates the horizontal scroll percentage of the 'scrollElement'.
 * Updates the 'scrollPercentage' ref with the calculated value.
 */
function updateScroll() {
  let scrollLeft: number; // Stores the current horizontal scroll position (pixels from the left).
  let scrollWidth: number; // Stores the total scrollable width minus the visible width.

  // Check if the scroll element is a specific HTMLElement or the window.
  if (scrollElement instanceof HTMLElement) {
    // If it's an HTMLElement, use its specific properties.
    scrollLeft = scrollElement.scrollLeft;
    // scrollWidth is the total width of the content.
    // clientWidth is the visible width of the element.
    // The difference is the maximum amount you can scroll horizontally.
    scrollWidth = scrollElement.scrollWidth - scrollElement.clientWidth;
  } else {
    // If it's the window, use window/document properties.
    // window.scrollX is the horizontal scroll position of the window.
    scrollLeft = window.scrollX;
    // document.documentElement.scrollWidth is the total width of the document.
    // window.innerWidth is the visible width of the viewport.
    scrollWidth = document.documentElement.scrollWidth - window.innerWidth;
  }

  // Calculate the percentage.
  // Check if scrollWidth > 0 to avoid division by zero if the element is not horizontally scrollable.
  scrollPercentage.value = scrollWidth > 0
    ? Math.round((scrollLeft / scrollWidth) * 100) // Calculate percentage if scrollable
    : 0; // Default to 0% if not scrollable
}

/**
 * Adds the scroll event listener to the current 'scrollElement'.
 * It correctly adds the listener to either an HTMLElement or the window object.
 * Also calls updateScroll once immediately to set the initial percentage.
 */
function setupScrollListener() {
  if (scrollElement instanceof HTMLElement) {
    scrollElement.addEventListener('scroll', updateScroll, { passive: true }); // Use passive for potential performance improvement
  } else {
    window.addEventListener('scroll', updateScroll, { passive: true });
  }
  // Update scroll percentage immediately after setup
  updateScroll();
}

/**
 * Removes the scroll event listener from the current 'scrollElement'.
 * This is crucial for cleanup to prevent memory leaks when the component
 * is unmounted or when the target element changes.
 */
function removeScrollListener() {
  if (scrollElement instanceof HTMLElement) {
    scrollElement.removeEventListener('scroll', updateScroll);
  } else {
    window.removeEventListener('scroll', updateScroll);
  }
}

// --- Lifecycle Hooks ---
/**
 * Lifecycle hook: Runs when the component is first mounted to the DOM.
 */
onMounted(() => {
  // If a targetRef prop was provided during component initialization,
  // set it as the element to track. Otherwise, scrollElement remains 'window'.
  if (props.targetRef) {
    scrollElement = props.targetRef;
  }
  // Add the scroll event listener to the determined scrollElement.
  setupScrollListener();
});

/**
 * Lifecycle hook: Runs just before the component is unmounted from the DOM.
 */
onUnmounted(() => {
  // Clean up: Remove the scroll event listener to prevent memory leaks.
  removeScrollListener();
});

// --- Watchers ---
/**
 * Watch for changes in the 'targetRef' prop.
 * This allows the component to react if the target element changes after mounting.
 */
watch(() => props.targetRef, (newTargetRef) => {
  // When targetRef changes:
  // 1. Remove the listener from the *old* scrollElement.
  removeScrollListener();
  // 2. Update scrollElement: Use the new targetRef if provided, otherwise default back to window.
  scrollElement = newTargetRef || window;
  // 3. Set up the listener on the *new* scrollElement.
  setupScrollListener();
});
</script>

<template>
  <div class="fixed bottom-4 left-1/2 transform -translate-x-1/2 bg-blue-600 text-white px-3 py-1 rounded shadow">
    Horizontal Scroll: {{ scrollPercentage }}%
  </div>
</template>

<style scoped>
/* Scoped styles: These CSS rules only apply to elements within this component */
div {
  /* Add a smooth transition for opacity changes (though opacity isn't changed here currently) */
  transition: opacity 0.3s ease;
  /* Ensure the element is interactive and doesn't block clicks on elements underneath */
  pointer-events: none;
  /* Improve text rendering */
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
</style>