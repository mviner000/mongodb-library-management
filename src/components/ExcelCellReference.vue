<!-- ExcelCellReference.vue -->
<template>
  <div class="fixed h-[42px] top-14 left-0 z-40 w-full flex items-center bg-white border-b border-b-gray-400">
    <!-- Cell reference box (e.g., A1) -->
    <div class="flex items-center px-2">
      <div class="flex items-center cursor-pointer">
        <span class="text-sm font-bold text-gray-700">{{ cellReference }}</span>
      </div>
    </div>

    <!-- fx formula indicator -->
    <div class="flex items-center px-3 text-gray-500">
      <span class="text-sm italic">fx</span>
    </div>

    <!-- Empty space -->
    <div class="flex-1 h-full"></div>
    
    <!-- Delete button (right-aligned, smaller size) -->
    <div class="mr-4">
      <button class="flex items-center justify-center px-3 py-1 text-xs rounded-md bg-red-100 text-red-500 border border-red-300 hover:bg-red-200">
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="mr-1">
          <path d="M3 6h18"></path>
          <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
          <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
        </svg>
        Delete
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { computed } from 'vue';

  const props = defineProps<{
    selectedCell: { colIndex: number; rowNumber: number } | null;
  }>();
  
  const getColumnLabel = (index: number): string => {
    let label = '';
    let i = index;
    do {
      const remainder = i % 26;
      label = String.fromCharCode(65 + remainder) + label;
      i = Math.floor(i / 26) - 1;
    } while (i >= 0);
    return label;
  };
  
  const cellReference = computed(() => {
    if (!props.selectedCell) return 'A1'; // Default when no cell is selected
    const { colIndex, rowNumber } = props.selectedCell;
    return `${getColumnLabel(colIndex)}${rowNumber}`;
  });
</script>