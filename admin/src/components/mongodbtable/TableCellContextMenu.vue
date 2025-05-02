<script setup lang="ts">
  import { ref, PropType } from 'vue'

  // Props definition
  const props = defineProps({
    isVisible: {
      type: Boolean,
      required: true,
    },
    position: {
      type: Object as PropType<{ x: number; y: number }>,
      required: true,
    },
    isPinned: {
      type: Boolean,
      default: false,
    },
    isArchived: {
      type: Boolean,
      default: false,
    },
    isPreview: {
      type: Boolean,
      default: false,
    },
  })

  // Emits definition
  const emit = defineEmits<{
    (e: 'close'): void
    (e: 'pin'): void
    (e: 'bookmark'): void
  }>()

  // Local state for tooltip
  const showPinTooltip = ref(false)

  // Methods to emit events
  const handleClose = () => {
    emit('close')
  }

  const handlePin = () => {
    if (!props.isArchived) {
      emit('pin')
    }
  }

  const handleBookmark = () => {
    emit('bookmark')
  }

  // Note: The parent component (MongoDBDataTable.vue) already has a global click listener
  // that calls `closeContextMenu`. We just need to ensure clicks *inside* the menu
  // don't propagate and trigger the parent's listener unintentionally.
  // We achieve this by calling `handleClose` which emits 'close' on item clicks.
  // Or, simply let the parent handle closing via its existing mechanism.
  // The @click="handleClose" on the main div ensures clicking the *background* of the menu closes it.
</script>

<template>
  <div
    v-if="isVisible && !isPreview"
    class="fixed z-50 bg-white shadow-lg border rounded-md p-1 min-w-[120px] context-menu"
    :style="{
      left: `${position.x}px`,
      top: `${position.y}px`,
    }"
    @click.stop="handleClose"
  >
    <div
      class="flex items-center px-3 py-1.5 text-sm rounded-sm relative tooltip-container"
      :class="[
        isArchived
          ? 'text-gray-400 cursor-not-allowed'
          : 'hover:bg-gray-100 cursor-pointer text-gray-700',
      ]"
      @click.stop="handlePin"
      @mouseenter="isArchived && (showPinTooltip = true)"
      @mouseleave="showPinTooltip = false"
    >
      <span>
        <template v-if="isPinned">📌 Unpin this item</template>

        <template v-else>📌 Pin this item</template>
      </span>
      <div
        v-if="isArchived && showPinTooltip"
        class="custom-tooltip absolute bg-gray-800 text-white text-xs rounded py-1 px-2 left-0 bottom-full mb-1 whitespace-nowrap pointer-events-none z-50"
      >
        You cannot pin an archived item
        <div
          class="tooltip-arrow absolute top-full left-4 w-2 h-2 bg-gray-800 transform rotate-45"
        ></div>
      </div>
    </div>
    <div
      class="flex items-center px-3 py-1.5 text-sm hover:bg-gray-100 rounded-sm cursor-pointer"
      @click.stop="handleBookmark"
    >
      🔖 Bookmark
    </div>
  </div>
</template>

<style scoped>
  /* Context Menu */
  .tooltip-container {
    position: relative;
  }

  .custom-tooltip {
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
    animation: fadeIn 0.2s ease-in-out;
  }

  .tooltip-arrow {
    position: absolute;
    top: 100%;
    left: 10px;
    margin-top: -4px;
    width: 8px;
    height: 8px;
    transform: rotate(45deg);
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .excel-cell-context-selected {
    outline: 2px solid #217346;
    outline-offset: -2px;
    position: relative;
    z-index: 1;
    overflow: visible;
  }

  .context-menu {
    transform: translateY(-100%); /* Move menu up by its own height */
    pointer-events: auto; /* Ensure menu remains interactive */
  }

  /* small arrow */
  .context-menu::before {
    content: '';
    position: absolute;
    top: -5px; /* Change from bottom to top */
    left: 10px;
    width: 0;
    height: 0;
    border-left: 5px solid transparent;
    border-right: 5px solid transparent;
    border-bottom: 5px solid white; /* Change from border-top to border-bottom */
    filter: drop-shadow(0 -1px 1px rgba(0, 0, 0, 0.1)); /* Adjust shadow direction */
  }
</style>
