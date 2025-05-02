<!-- src/components/sidebar/SettingsNavItem.vue -->
<template>
    <div class="flex flex-col">
      <div
        @click="handleSettingsClick"
        class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg cursor-pointer"
      >
        <svg 
          xmlns="http://www.w3.org/2000/svg" 
          width="20" 
          height="20" 
          viewBox="0 0 24 24" 
          fill="none"
          stroke="currentColor" 
          stroke-width="2" 
          stroke-linecap="round" 
          stroke-linejoin="round"
          class="text-gray-600"
        >
          <circle cx="12" cy="12" r="3"></circle>
          <path
            d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 
            1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 
            2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 
            2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 
            2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 
            2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 
            2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 
            2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
        </svg>
        <span class="text-sm">Settings</span>
        <span class="ml-auto text-sm">{{ settingsUnlocked ? '🔓' : '🔒' }}</span>
      </div>
      
      <!-- PIN Input Dialog uses shadcn Dialog now -->
      <PinInputDialog
        v-model:visible="showPinModal"
        :correct-pin="settingsPin"
        title="Unlock Settings"
        :initially-show-pin="showPinDigits"
        @correct="handlePinSuccess"
        @cancel="handlePinCancel"
      />
    </div>
  </template>
  
  <script setup lang="ts">
  import { ref, onMounted } from 'vue';
  import { useRouter } from 'vue-router';
  import PinInputDialog from '@/components/PinInputDialog.vue';
  
  const router = useRouter();
  
  // PIN configuration
  const showPinModal = ref(false);
  const settingsPin = ref('123456'); // Set the PIN to 123456 as requested
  const showPinDigits = ref(false);
  
  // Settings lock state
  const settingsUnlocked = ref(false);
  
  // Check if settings are unlocked
  const checkSettingsStatus = () => {
    const storedTime = sessionStorage.getItem('settingsUnlocked');
    if (!storedTime) {
      settingsUnlocked.value = false;
      return;
    }
  
    const unlockTime = parseInt(storedTime, 10);
    const currentTime = Date.now();
    // Settings unlock expires after 1 hour (3600000 ms)
    settingsUnlocked.value = (currentTime - unlockTime) <= 3600000;
  };
  
  // Handle settings click
  const handleSettingsClick = () => {
    checkSettingsStatus(); // Check current lock status
    
    if (!settingsUnlocked.value) {
      // Show PIN modal if settings are locked
      showPinModal.value = true;
      return;
    }
    
    // Navigate to settings if already unlocked
    navigateToSettings();
  };
  
  // Navigate to settings page
  const navigateToSettings = () => {
    router.replace('/settings');
  };
  
  // Handle successful PIN entry
  const handlePinSuccess = (enteredPin: string) => {
    console.log('Correct PIN entered:', enteredPin);
    
    // Store unlock time in session storage
    const currentTime = Date.now();
    sessionStorage.setItem('settingsUnlocked', currentTime.toString());
    
    // Update state and navigate
    settingsUnlocked.value = true;
    showPinModal.value = false;
    navigateToSettings();
  };
  
  // Handle PIN entry cancellation
  const handlePinCancel = () => {
    console.log('PIN entry cancelled');
    // The v-model:visible binding will handle closing the modal
  };
  
  // Check lock status on component mount
  onMounted(() => {
    checkSettingsStatus();
  });
  </script>