<!-- src/components/FooterTabsBar.vue -->
<template>
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
          
          <div class="sheet-tab">
            <span class="sheet-name">Recoveries</span>
            <div class="sheet-dropdown">
              <span class="dropdown-icon">▼</span>
            </div>
          </div>
          
          <div class="sheet-tab">
            <span class="sheet-name">Archives</span>
            <div class="sheet-dropdown">
              <span class="dropdown-icon">▼</span>
            </div>
          </div>
          
          <div class="sheet-tab">
            <span class="sheet-name">All</span>
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
  
  <script>
  export default {
    name: 'SheetsTabsBar'
  };
  </script>
  
  <style scoped>
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