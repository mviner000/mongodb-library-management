<!-- src/components/AlphabeticalHeader.vue -->
<template>
    <div class="excel-header-row flex">
      <div 
        v-for="(letter, index) in headerLetters" 
        :key="index"
        class="excel-header-cell min-w-[100px] text-center border border-gray-300 bg-gray-100 p-1 font-medium"
      >
        {{ letter }}
      </div>
    </div>
  </template>
  
  <script setup>
  import { computed } from 'vue';
  
  // Accept number of columns as prop
  const props = defineProps({
    columnCount: {
      type: Number,
      default: 0
    }
  });
  
  // Generate alphabetical column headers (A, B, C, ... Z, AA, AB, etc.)
  const headerLetters = computed(() => {
    const letters = [];
    const baseChars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    
    for (let i = 0; i < props.columnCount; i++) {
      if (i < 26) {
        // First 26 columns are A-Z
        letters.push(baseChars[i]);
      } else {
        // After Z, use AA, AB, etc.
        const firstChar = baseChars[Math.floor((i - 26) / 26)];
        const secondChar = baseChars[(i - 26) % 26];
        letters.push(firstChar + secondChar);
      }
    }
    
    return letters;
  });
  </script>
  
  <style scoped>
  .excel-header-row {
    border-bottom: none;
  }
  
  .excel-header-cell {
    flex: 1;
    min-width: 100px;
  }
  </style>