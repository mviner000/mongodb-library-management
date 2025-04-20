<!-- src/views/ExcelCellReference.vue -->
<script setup lang="ts">
  import { ArrowUpToLine } from 'lucide-vue-next'

  defineProps<{
    selectedCell: { colIndex: number; rowNumber: number } | null
    selectedRows: Set<string>
    isSidebarOpen: boolean
    previewMode?: boolean
  }>()

  const emit = defineEmits<{
    (e: 'reset-selection'): void
  }>()

  // Placeholder function for recovery
  const handleCSVImport = () => {
    emit('reset-selection')
  }
</script>

<template>
  <!-- ExcelCellReference main div -->
  <div
    class="fixed h-[42px] z-30 top-14 w-screen flex items-center bg-white border-b border-b-gray-400 transition-all duration-300 ease-in-out"
    :class="[isSidebarOpen ? 'left-[280px]' : 'left-0', { hidden: previewMode }]"
  >
    <!-- Cell reference box (e.g., CSV) -->
    <div class="flex items-center px-2">
      <div class="flex items-center cursor-pointer">
        <span class="text-sm font-bold text-gray-700">CSV</span>
      </div>
    </div>

    <!-- fx formula indicator -->
    <div class="flex items-center px-3 text-gray-500">
      <span class="text-sm italic">fx</span>
    </div>

    <!-- Empty space -->
    <div class="flex-1 h-full"></div>
    <div
      class="flex transition-all duration-300 ease-in-out"
      :class="isSidebarOpen ? 'mr-[280px]' : 'mr-[0px]'"
    >
      <!-- single selection button -->
      <div
        v-if="selectedRows.size === 1"
        class="mr-5 flex gap-2"
      >
        <!-- Single CSVImport Button -->
        <button
          @click="handleCSVImport"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-green-100 text-green-600 border-green-300 hover:bg-green-200"
        >
          <ArrowUpToLine class="h-3 w-3 mr-1" />
          CSVImport 1 Item
        </button>
      </div>

      <!-- buttons for multiple selections -->
      <div
        v-if="selectedRows.size > 1"
        class="mr-5 flex gap-2"
      >
        <!-- Batch CSVImport Button -->
        <button
          @click="handleCSVImport"
          class="flex items-center justify-center px-3 py-1 text-xs rounded-md border bg-green-100 text-green-600 border-green-300 hover:bg-green-200"
        >
          <ArrowUpToLine class="h-3 w-3 mr-1" />
          Batch CSVImport {{ selectedRows.size }} Items
        </button>
      </div>
    </div>
  </div>
  <div class="sheets-tabs-container">
    <div class="sheets-tabs-wrapper">
      <button class="nav-button">
        <span class="nav-icon">◀</span>
      </button>

      <div class="sheets-tabs">
        <div class="sheet-tab active">
          <span class="sheet-name">Default</span>
          <div class="sheet-dropdown">
            <span class="dropdown-icon">▼</span>
          </div>
        </div>
      </div>

      <button class="nav-button add-sheet">
        <span class="add-icon">+</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
  .custom-delete-dialog {
    max-width: 500px;
    border-radius: 4px;
  }

  /* Keep button styles within the dialog consistent */
  .bg-rose-100 {
    background-color: #ffebee;
  }
  .text-rose-700 {
    color: #c6282d;
  }
  .border-rose-200 {
    border-color: #ffcdd2;
  }
  .text-rose-600 {
    color: #e53935;
  }
  .bg-rose-600 {
    background-color: #e53935;
  }
  .hover\:bg-rose-700:hover {
    background-color: #d32f2f;
  }

  /* for footer tab bar */
  .sheets-tabs-container {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    background-color: #f1f3f4;
    border-top: 1px solid #dadce0;
    z-index: 1000;
  }

  .sheets-tabs-wrapper {
    display: flex;
    align-items: center;
    height: 36px;
    padding: 0 4px;
  }

  .sheets-tabs {
    display: flex;
    overflow-x: auto;
    scrollbar-width: none; /* Hide scrollbar for Firefox */
    -ms-overflow-style: none; /* Hide scrollbar for IE and Edge */
  }

  .sheets-tabs::-webkit-scrollbar {
    display: none; /* Hide scrollbar for Chrome, Safari and Opera */
  }

  .sheet-tab {
    display: flex;
    align-items: center;
    min-width: 60px;
    max-width: 180px;
    height: 28px;
    margin: 4px 2px;
    padding: 0 8px;
    border-radius: 4px 4px 0 0;
    background-color: #e0e0e0;
    color: #5f6368;
    font-size: 13px;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition:
      background-color 0.2s,
      color 0.2s,
      border-bottom 0.2s;
  }

  .sheet-tab.active {
    background-color: white;
    color: #1a73e8;
    border-bottom: 2px solid #1a73e8;
  }

  .sheet-name {
    flex-grow: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sheet-dropdown {
    position: relative;
    margin-left: 4px;
    padding: 2px;
  }

  .dropdown-icon {
    font-size: 8px;
    color: #5f6368;
  }

  .nav-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    margin: 0 2px;
    border: none;
    border-radius: 50%;
    background-color: transparent;
    cursor: pointer;
  }

  .nav-button:hover {
    background-color: #e0e0e0;
  }

  .add-sheet {
    color: #5f6368;
  }

  .add-icon {
    font-size: 18px;
  }

  .nav-icon {
    font-size: 12px;
    color: #5f6368;
  }
</style>
