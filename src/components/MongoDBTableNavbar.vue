<!-- src/components/MongoDBTableNavbar.vue -->
<script setup lang="ts">
import { useRouter, useRoute } from 'vue-router';
import { Avatar, AvatarImage, AvatarFallback } from '@/components/ui/avatar';
import { Button } from '@/components/ui/button';
import { ref, watch } from 'vue'; // Add watch

// Define the Tab interface
// interface Tab {
//   id: string;
//   title: string;
//   type: 'home' | 'collection' | 'hello';
//   path: string;
//   collectionName?: string;
//   reloadCount: number;
// }

// Accept both direct prop and route param
const props = defineProps<{
  selectedCollection?: string;
  name?: string; // From route params
  title?: string;
  isSplitActive?: boolean; // Prop to determine if split mode is active
}>();

// Define the TabManager interface
// interface TabManager {
//   openNewTab: (tab: Tab) => void;
//   addNewTab: () => void;
//   closeTab: (tabId: string) => void;
//   getActiveTabId: () => string;
//   getTab: (tabId: string) => Tab | undefined;
// }

const router = useRouter();
const route = useRoute(); // Add this to access route params


// Use route param or direct prop, with fallback to 'users'
const collectionName = ref(props.name || props.selectedCollection || 'users');

// Watch for route changes to update collection name
watch(() => route.params.name, (newVal) => {
  if (newVal && newVal !== collectionName.value) {
    collectionName.value = newVal as string;
  }
}, { immediate: true });

// Watch for prop changes from parent
watch(() => props.selectedCollection, (newVal) => {
  if (newVal && newVal !== collectionName.value) {
    collectionName.value = newVal;
  }
});

const navigateToHome = () => {
  router.replace('/home');
};

const menuItems = [
  'File', 'Edit', 'View', 'Insert', 'Format', 'Data', 'Tools', 'Extensions', 'Help'
];

// Expose setCollection method to parent components
defineExpose({
  setCollection: (name: string) => {
    collectionName.value = name;
  }
});
</script>

<template>
  <header class="sticky top-0 z-0 flex items-center w-full h-14 px-2 border-b bg-white">
    <!-- Left section: Menu and title -->
    <div class="flex items-center gap-2">
        
      <Button variant="ghost" size="icon" class="text-gray-500">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="12" x2="21" y2="12"></line>
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="3" y1="18" x2="21" y2="18"></line>
        </svg>
      </Button>

      <!-- Only show Home button when not in split mode -->
      <Button 
          v-if="!isSplitActive"
          variant="ghost" 
          size="sm" 
          @click="navigateToHome"
          class="text-gray-500 hover:text-gray-800"
        >
        Home
      </Button>
      {{ collectionName }}

      
      
      <div class="flex items-center gap-2">
        <span class="font-medium text-lg text-gray-800">{{ title }}</span>
      </div>
      
      <nav class="ml-4 flex items-center gap-1">
        <Button 
          v-for="item in menuItems" 
          :key="item"
          variant="ghost" 
          class="text-sm px-2 h-8 hover:bg-gray-100 rounded-sm"
        >
          {{ item }}
        </Button>
      </nav>
    </div>

    <!-- Right section: User profile -->
    <div class="flex items-center gap-2 ml-auto">
      <div class="flex items-center gap-1 ml-2">
        <div class="flex -space-x-2">
          <Avatar class="h-8 w-8 border-2 border-white">
            <AvatarFallback class="text-xs">GJC</AvatarFallback>
          </Avatar>
          <Avatar class="h-8 w-8 border-2 border-white">
            <AvatarImage src="" alt="User" />
            <AvatarFallback class="bg-gray-200">
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <circle cx="12" cy="8" r="5" />
                <path d="M20 21a8 8 0 0 0-16 0" />
              </svg>
            </AvatarFallback>
          </Avatar>
        </div>
      </div>
    </div>
  </header>
</template>