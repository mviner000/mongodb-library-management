<!-- src/components/HistoryPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useToast } from '../components/ui/toast';

interface HistoryItem {
  tabId: string;
  nameOfTheOpenedLink: string;
  created_at: string;
  urlLink: string;
}

const router = useRouter();
const historyItems = ref<HistoryItem[]>([]);
const sortNewestFirst = ref(true);
const searchQuery = ref('');
const { toast } = useToast();

// For delete confirmation dialog
const showDeleteDialog = ref(false);

// Group by date functionality
const groupedHistory = ref<Record<string, HistoryItem[]>>({});
const viewMode = ref('date'); // 'date' or 'group'

onMounted(() => {
  loadHistory();
});

function loadHistory() {
  const historyData = sessionStorage.getItem('browserHistory');
  if (historyData) {
    historyItems.value = JSON.parse(historyData);
    sortHistory();
    groupHistoryByDate();
  }
}

function sortHistory() {
  if (sortNewestFirst.value) {
    // Sort by newest first (descending order of created_at)
    historyItems.value.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime());
  } else {
    // Sort by oldest first (ascending order of created_at)
    historyItems.value.sort((a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime());
  }
}

function toggleSortOrder() {
  sortNewestFirst.value = !sortNewestFirst.value;
  sortHistory();
  groupHistoryByDate();
}

function formatTimestamp(isoString: string) {
  const date = new Date(isoString);
  return date.toLocaleTimeString('en-US', {
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
  });
}

function formatDate(isoString: string) {
  const date = new Date(isoString);
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);
  
  if (date.toDateString() === today.toDateString()) {
    return "Today - " + date.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' });
  } else if (date.toDateString() === yesterday.toDateString()) {
    return "Yesterday - " + date.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' });
  } else {
    return date.toLocaleDateString('en-US', { weekday: 'long', month: 'long', day: 'numeric', year: 'numeric' });
  }
}

function groupHistoryByDate() {
  const grouped: Record<string, HistoryItem[]> = {};
  
  historyItems.value.forEach(item => {
    const date = new Date(item.created_at);
    const dateStr = date.toDateString();
    
    if (!grouped[dateStr]) {
      grouped[dateStr] = [];
    }
    
    grouped[dateStr].push(item);
  });
  
  groupedHistory.value = grouped;
}

function navigateToUrl(urlLink: string) {
  const path = urlLink.replace(/^app\//, '');
  router.push(`/${path}`);
}

function switchView(mode: string) {
  viewMode.value = mode;
}

function showDeleteConfirmation() {
  showDeleteDialog.value = true;
}

function confirmDeleteBrowsingData() {
  // Clear browsing history from session storage
  sessionStorage.removeItem('browserHistory');
  
  // Clear local arrays and objects
  historyItems.value = [];
  groupedHistory.value = {};
  
  // Close the dialog
  showDeleteDialog.value = false;
  
  // Show toast notification
  toast({
    title: "Browsing Data Deleted",
    description: "Your browsing history has been cleared.",
  });
}

function cancelDeleteBrowsingData() {
  showDeleteDialog.value = false;
}
</script>

<template>
  <div class="history-page">
    <div class="history-container">
      <!-- Left sidebar -->
      <div class="sidebar">
        <div class="sidebar-item active">
          <svg class="sidebar-icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M13 3c-4.97 0-9 4.03-9 9H1l3.89 3.89.07.14L9 12H6c0-3.87 3.13-7 7-7s7 3.13 7 7-3.13 7-7 7c-1.93 0-3.68-.79-4.94-2.06l-1.42 1.42C8.27 19.99 10.51 21 13 21c4.97 0 9-4.03 9-9s-4.03-9-9-9zm-1 5v5l4.25 2.52.77-1.28-3.52-2.09V8z"></path>
          </svg>
          <span>Chrome history</span>
        </div>
        <div class="sidebar-item">
          <svg class="sidebar-icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M21 3H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H3V5h18v14zM8 15h8v2H8z"></path>
          </svg>
          <span>Tabs from other devices</span>
        </div>
        <div class="sidebar-item" @click="showDeleteConfirmation">
          <svg class="sidebar-icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"></path>
          </svg>
          <span>Delete browsing data</span>
          <svg class="external-link-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
            <path d="M19 19H5V5h7V3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2v-7h-2v7zM14 3v2h3.59l-9.83 9.83 1.41 1.41L19 6.41V10h2V3h-7z"></path>
          </svg>
        </div>
      </div>

      <!-- Main content area -->
      <div class="main-content">
        <div class="header">
          <h1 class="title">History</h1>
          <div class="controls">
            <div class="search-box">
              <svg class="search-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor">
                <path d="M15.5 14h-.79l-.28-.27C15.41 12.59 16 11.11 16 9.5 16 5.91 13.09 3 9.5 3S3 5.91 3 9.5 5.91 16 9.5 16c1.61 0 3.09-.59 4.23-1.57l.27.28v.79l5 4.99L20.49 19l-4.99-5zm-6 0C7.01 14 5 11.99 5 9.5S7.01 5 9.5 5 14 7.01 14 9.5 11.99 14 9.5 14z"></path>
              </svg>
              <input type="text" placeholder="Search history" v-model="searchQuery" class="search-input">
            </div>
            <div class="view-controls">
              <button class="view-button" :class="{ active: viewMode === 'date' }" @click="switchView('date')">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="view-icon">
                  <path d="M3 13h2v-2H3v2zm0 4h2v-2H3v2zm0-8h2V7H3v2zm4 4h14v-2H7v2zm0 4h14v-2H7v2zM7 7v2h14V7H7z"></path>
                </svg>
                By date
              </button>
              <button class="view-button" :class="{ active: viewMode === 'group' }" @click="switchView('group')">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="view-icon">
                  <path d="M3 9h4V5H3v4zm0 5h4v-4H3v4zm0 5h4v-4H3v4zm5 0h4v-4H8v4zm5 0h4v-4h-4v4zM8 5v4h4V5H8zm5 0v4h4V5h-4zm5 5h4v-4h-4v4zm0 5h4v-4h-4v4z"></path>
                </svg>
                By group
              </button>
            </div>
          </div>
        </div>

        <!-- History items grouped by date -->
        <div class="history-list" v-if="viewMode === 'date'">
          <div v-for="(items, dateStr) in groupedHistory" :key="dateStr" class="date-group">
            <div class="date-header">{{ formatDate(new Date(dateStr).toISOString()) }}</div>
            <div v-for="(item, index) in items" :key="index" class="history-item">
              <input type="checkbox" class="checkbox">
              <div class="time">{{ formatTimestamp(item.created_at) }}</div>
              <div class="favicon">
                <img src="https://www.google.com/favicon.ico" alt="favicon" class="favicon-img"
                  v-if="item.urlLink.includes('google.com')">
                <div class="default-favicon" v-else>🌐</div>
              </div>
              <div class="item-details" @click="navigateToUrl(item.urlLink)">
                <div class="item-title">{{ item.nameOfTheOpenedLink }}</div>
                <div class="item-url">{{ item.urlLink.replace('app/', '') }}</div>
              </div>
              <button class="more-button">
                <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="more-icon">
                  <path d="M12 8c1.1 0 2-.9 2-2s-.9-2-2-2-2 .9-2 2 .9 2 2 2zm0 2c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2zm0 6c-1.1 0-2 .9-2 2s.9 2 2 2 2-.9 2-2-.9-2-2-2z"></path>
                </svg>
              </button>
            </div>
          </div>
        </div>

        <!-- History items grouped by other criteria (placeholder for 'By group' view) -->
        <div class="history-list" v-else>
          <!-- Placeholder for group view -->
          <div class="empty-state">
            Group view is under development
          </div>
        </div>
      </div>
    </div>

    <!-- Delete Confirmation Dialog -->
    <div v-if="showDeleteDialog" class="dialog-overlay">
      <div class="dialog-content">
        <h3 class="dialog-title">Delete browsing data</h3>
        <p class="dialog-description">
          Are you sure you want to delete all browsing history? This action cannot be undone.
        </p>
        <div class="dialog-actions">
          <button class="dialog-btn cancel" @click="cancelDeleteBrowsingData">Cancel</button>
          <button class="dialog-btn delete" @click="confirmDeleteBrowsingData">Delete</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.history-page {
  background-color: #202124;
  color: white;
  font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.history-container {
  display: flex;
  height: 100vh;
}

/* Sidebar styles */
.sidebar {
  width: 256px;
  background-color: #202124;
  border-right: 1px solid #3c4043;
  padding: 12px 0;
  flex-shrink: 0;
}

.sidebar-item {
  display: flex;
  align-items: center;
  padding: 8px 20px;
  color: #e8eaed;
  cursor: pointer;
  position: relative;
}

.sidebar-item:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.sidebar-item.active {
  background-color: rgba(138, 180, 248, 0.12);
  color: #8ab4f8;
}

.sidebar-icon {
  width: 20px;
  height: 20px;
  margin-right: 16px;
}

.external-link-icon {
  width: 16px;
  height: 16px;
  position: absolute;
  right: 20px;
}

/* Main content styles */
.main-content {
  flex: 1;
  padding: 0;
  overflow-y: auto;
}

.header {
  padding: 20px;
  position: sticky;
  top: 0;
  background-color: #202124;
  z-index: 10;
}

.title {
  font-size: 22px;
  font-weight: normal;
  margin-bottom: 16px;
}

.controls {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.search-box {
  position: relative;
  flex: 1;
  max-width: 600px;
}

.search-icon {
  position: absolute;
  left: 12px;
  top: 50%;
  transform: translateY(-50%);
  width: 18px;
  height: 18px;
  color: #9aa0a6;
}

.search-input {
  width: 100%;
  padding: 8px 8px 8px 40px;
  background-color: #303134;
  border: none;
  border-radius: 100px;
  color: white;
  font-size: 14px;
}

.search-input:focus {
  outline: none;
  background-color: #3c4043;
}

.view-controls {
  display: flex;
  gap: 8px;
}

.view-button {
  display: flex;
  align-items: center;
  padding: 8px 16px;
  background-color: transparent;
  border: none;
  color: #e8eaed;
  border-radius: 4px;
  cursor: pointer;
}

.view-button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.view-button.active {
  background-color: rgba(138, 180, 248, 0.12);
  color: #8ab4f8;
}

.view-icon {
  width: 18px;
  height: 18px;
  margin-right: 8px;
}

/* History list styles */
.history-list {
  padding: 0 20px 20px;
}

.date-group {
  margin-bottom: 24px;
}

.date-header {
  font-size: 14px;
  color: #e8eaed;
  padding: 8px 0;
  border-bottom: 1px solid #3c4043;
  margin-bottom: 8px;
}

.history-item {
  display: flex;
  align-items: center;
  padding: 8px;
  border-radius: 4px;
}

.history-item:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.checkbox {
  margin-right: 16px;
}

.time {
  width: 80px;
  font-size: 12px;
  color: #9aa0a6;
}

.favicon {
  width: 20px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 16px;
}

.favicon-img {
  width: 16px;
  height: 16px;
}

.default-favicon {
  font-size: 16px;
}

.item-details {
  flex: 1;
  min-width: 0;
  cursor: pointer;
}

.item-title {
  font-size: 14px;
  color: #e8eaed;
  margin-bottom: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.item-url {
  font-size: 12px;
  color: #9aa0a6;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.more-button {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: #9aa0a6;
  border-radius: 50%;
  cursor: pointer;
}

.more-button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.more-icon {
  width: 20px;
  height: 20px;
}

.empty-state {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 200px;
  color: #9aa0a6;
}

/* Delete Dialog Styles */
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 50;
}

.dialog-content {
  background-color: #292a2d;
  border-radius: 8px;
  padding: 20px;
  width: 400px;
  max-width: 90%;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.dialog-title {
  font-size: 18px;
  margin: 0 0 12px 0;
  color: #e8eaed;
}

.dialog-description {
  color: #9aa0a6;
  font-size: 14px;
  margin-bottom: 20px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.dialog-btn {
  padding: 8px 16px;
  border-radius: 4px;
  font-size: 14px;
  cursor: pointer;
  border: none;
}

.dialog-btn.cancel {
  background-color: transparent;
  color: #8ab4f8;
}

.dialog-btn.cancel:hover {
  background-color: rgba(138, 180, 248, 0.1);
}

.dialog-btn.delete {
  background-color: #8ab4f8;
  color: #202124;
}

.dialog-btn.delete:hover {
  background-color: #699ef5;
}
</style>