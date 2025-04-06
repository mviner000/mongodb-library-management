<!-- src/components/TemplateGalleryNavbar.vue -->
<script setup lang="ts">
import { ref, watch } from 'vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import UserAvatarNavigation from '@/components/UserAvatarNavigation.vue';

const props = defineProps<{
  showSearch?: boolean;
  title?: string;
}>();

// Default title with reactive binding to props
const title = ref(props.title || 'Library Manager');

// Track search visibility
const showSearch = ref(props.showSearch !== true);

// Watch for title prop changes
watch(() => props.title, (newTitle) => {
  if (newTitle) {
    title.value = newTitle;
  }
});

// Watch for search visibility prop changes
watch(() => props.showSearch, (newValue) => {
  if (newValue !== undefined) {
    showSearch.value = newValue;
  }
});
</script>

<template>
  <header class="fixed top-0 z-50 flex items-center w-full h-14 px-2 border-b bg-white">
    <!-- Left section: Menu, logo and title -->
    <div class="flex items-center gap-2">
      <Button variant="ghost" size="icon" class="text-gray-500" >
        <!-- Hamburger Menu Icon -->
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="12" x2="21" y2="12"></line>
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="3" y1="18" x2="21" y2="18"></line>
        </svg>
      </Button>
      
      <div class="flex items-center gap-2">
        <div class="w-6 h-6">
          <svg viewBox="0 0 24 24" width="24" height="24">
            <rect fill="#0F9D58" x="0" y="0" width="24" height="24" rx="4" />
            <rect fill="#FFFFFF" x="4" y="4" width="16" height="16" rx="2" />
            <rect fill="#0F9D58" x="6" y="6" width="12" height="2" />
            <rect fill="#0F9D58" x="6" y="10" width="12" height="2" />
            <rect fill="#0F9D58" x="6" y="14" width="12" height="2" />
          </svg>
        </div>
        <span class="font-medium text-lg text-gray-800">{{ title }}</span>
      </div>
    </div>

    <!-- Middle section: Search (conditionally rendered) -->
    <div v-if="showSearch" class="flex-1 mx-4">
      <div class="relative max-w-2xl mx-auto">
        <div class="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
          <!-- Search Icon -->
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-400">
            <circle cx="11" cy="11" r="8"></circle>
            <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
          </svg>
        </div>
        <Input 
          class="pl-10 py-1.5 bg-gray-50 rounded-full text-sm border-gray-300 focus:ring-blue-500 focus:border-blue-500 w-full"
          placeholder="Search"
        />
      </div>
    </div>
    <!-- Empty div for spacing when search is hidden -->
    <div v-else class="flex-1"></div>

    <!-- Right section: Using the separated UserAvatarNavigation component -->
    <UserAvatarNavigation />
  </header>
</template>