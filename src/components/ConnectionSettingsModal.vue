<!-- src/components/ConnectionSettingsModal.vue -->
<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/dialog';
import { useToast } from '@/components/ui/toast';
import { AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogTitle } from '@/components/ui/alert-dialog';
import { testConnection } from '@/utils/api';
import { LoaderCircle } from 'lucide-vue-next';

defineProps<{
  isOpen: boolean;
}>();

const emit = defineEmits(['close', 'save']);

const { toast } = useToast();
const ipAddress = ref('');
const currentIpAddress = ref('localhost');
const isValidIp = ref(true);
const showConfirmDialog = ref(false);
const connectionStatus = ref('❓'); // Default emoji
const isTestingConnection = ref(false);
const confirmationMessage = ref('');
const connectionSuccessful = ref(false);
const isSaveButtonLoading = ref(false);
const isSaveButtonDisabled = ref(false);

// IPv4 validation regex
const ipv4Regex = /^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;

onMounted(() => {
  // Load saved IP address from localStorage
  const savedIp = localStorage.getItem('API_BASE_IP');
  if (savedIp) {
    currentIpAddress.value = savedIp;
  }
  // Set initial value for the input field
  ipAddress.value = currentIpAddress.value;
});

const validateIp = () => {
  // Check if input is "localhost" or a valid IPv4 address
  isValidIp.value = ipAddress.value === "localhost" || ipv4Regex.test(ipAddress.value);
  return isValidIp.value;
};

const handleSave = async () => {
  if (!validateIp() || isSaveButtonDisabled.value) return;
  
  // Set button to loading state and disable it
  isSaveButtonLoading.value = true;
  isSaveButtonDisabled.value = true;
  
  // First test the connection before showing confirmation dialog
  await handleTestConnection();
  
  // Set confirmation message based on connection test result
  if (connectionSuccessful.value) {
    confirmationMessage.value = `✅ Connection Success to ${ipAddress.value}:3000. Are you sure you want to continue?`;
  } else {
    confirmationMessage.value = `❌ Connection Failed to ${ipAddress.value}:3000. Are you sure you want to continue?`;
  }
  
  // Show confirmation dialog
  showConfirmDialog.value = true;
  
  // Keep button in loading state until connection test is complete
  isSaveButtonLoading.value = false;
  
  // Set a timeout to re-enable the button after 2 seconds
  setTimeout(() => {
    isSaveButtonDisabled.value = false;
  }, 2000);
};

const confirmSave = () => {
  // Close confirmation dialog
  showConfirmDialog.value = false;
  
  // Save IP address to localStorage
  localStorage.setItem('API_BASE_IP', ipAddress.value);
  
  // Notify parent component
  emit('save', ipAddress.value);
  
  // Show success toast
  toast({
    title: 'Connection Settings Updated',
    description: `API Base URL set to ${ipAddress.value}:3000`,
  });
  
  // Close the modal
  emit('close');
};

const cancelSave = () => {
  // Close confirmation dialog without saving
  showConfirmDialog.value = false;
};

const handleEnterKey = (event: KeyboardEvent) => {
  if (event.key === 'Enter' && validateIp() && !isSaveButtonDisabled.value) {
    handleSave();
  }
};

const handleClose = () => {
  // Reset to current value
  ipAddress.value = currentIpAddress.value;
  // Close modal
  emit('close');
};

const handleTestConnection = async () => {
  if (!validateIp()) {
    toast({
      title: 'Invalid IP Address',
      description: 'Please enter a valid IP address before testing the connection.',
      variant: 'destructive'
    });
    return false;
  }

  connectionStatus.value = '⏳'; // Loading emoji
  isTestingConnection.value = true;
  
  // Temporarily set the IP in localStorage to test with the current input value
  const originalIp = localStorage.getItem('API_BASE_IP');
  localStorage.setItem('API_BASE_IP', ipAddress.value);
  
  try {
    const isConnected = await testConnection();
    connectionSuccessful.value = isConnected;
    
    if (isConnected) {
      connectionStatus.value = '✅'; // Success emoji
      toast({
        title: 'Connection Successful',
        description: `Successfully connected to ${ipAddress.value}:3000`,
      });
    } else {
      connectionStatus.value = '❌'; // Failure emoji
      toast({
        title: 'Connection Failed',
        description: `Could not connect to ${ipAddress.value}:3000`,
        variant: 'destructive'
      });
    }
    return isConnected;
  } catch (error) {
    connectionStatus.value = '❌'; // Failure emoji
    connectionSuccessful.value = false;
    toast({
      title: 'Connection Error',
      description: `Error testing connection: ${error}`,
      variant: 'destructive'
    });
    return false;
  } finally {
    // Restore the original IP address in localStorage
    if (originalIp) {
      localStorage.setItem('API_BASE_IP', originalIp);
    } else {
      localStorage.removeItem('API_BASE_IP');
    }
    isTestingConnection.value = false;
  }
};
</script>

<template>
  <Dialog :open="isOpen" @update:open="(value) => !value && handleClose()">
    <DialogContent class="sm:max-w-md">
      <DialogHeader>
        <DialogTitle>Connection Settings</DialogTitle>
      </DialogHeader>
      
      <div class="py-4">
        <Label for="ipAddress" class="mb-2 block">API Base URL IP Address</Label>
        <div class="flex flex-col space-y-2">
          <div class="flex items-center">
            <Button
              variant="outline"
              size="icon"
              class="mr-2"
              :disabled="isTestingConnection"
              @click="handleTestConnection"
              :title="'Test Connection'"
            >
              <span class="text-lg">{{ connectionStatus }}</span>
            </Button>
            <Input 
              id="ipAddress" 
              v-model="ipAddress" 
              @input="validateIp" 
              @keydown="handleEnterKey"
              :class="{ 'border-red-500': !isValidIp }"
              placeholder="Enter IPv4 address (e.g., 192.168.1.1)"
            />
            <span class="ml-2 text-sm text-gray-600">:3000</span>
          </div>
          
          <div v-if="!isValidIp" class="text-red-500 text-sm">
            Please enter a valid IPv4 address
          </div>
          
          <div class="text-sm text-gray-600">
            Current API Base URL: <span class="font-medium">{{ currentIpAddress }}:3000</span>
          </div>
        </div>
      </div>
      
      <DialogFooter class="flex justify-between">
        <Button variant="outline" @click="handleClose">Cancel</Button>
        <Button 
          :disabled="!isValidIp || ipAddress === '' || isSaveButtonDisabled"
          @click="handleSave"
          :class="{ 'opacity-50 cursor-not-allowed': isSaveButtonDisabled }"
        >
          <span v-if="isSaveButtonLoading" class="inline-block animate-spin"><LoaderCircle /></span>
          Save Settings
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
  
  <AlertDialog :open="showConfirmDialog" @update:open="(value) => !value && cancelSave()">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>Are you sure?</AlertDialogTitle>
        <AlertDialogDescription>
          {{ confirmationMessage }}
        </AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel @click="cancelSave">Cancel</AlertDialogCancel>
        <AlertDialogAction @click="confirmSave">Yes, save changes</AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>