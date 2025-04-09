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
    <div class="flex-1 h-full  "></div>
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