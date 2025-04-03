<!-- src/components/SheetsNavbar.vue -->
<script setup lang="ts">
import { ref, inject } from 'vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Avatar, AvatarImage, AvatarFallback } from '@/components/ui/avatar';

const title = ref('Library Manager');

// Get sidebar state from parent
const sidebarState = inject('sidebarState') as {
  isOpen: { value: boolean },
  toggle: () => void,
  close: () => void
}
</script>

<template>
  <header class="sticky top-0 z-50 flex items-center w-full h-14 px-2 border-b bg-white">
    <!-- Left section: Menu, logo and title -->
    <div class="flex items-center gap-2">
      <Button variant="ghost" size="icon" class="text-gray-500" @click="sidebarState.toggle()">
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

    <!-- Middle section: Search -->
    <div class="flex-1 mx-4">
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

    <!-- Right section: User profile and actions -->
    <div class="flex items-center gap-2">
      <Button variant="ghost" size="icon" class="text-gray-500">
        <!-- More Vertical Icon (3 dots) -->
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="1"></circle>
          <circle cx="12" cy="5" r="1"></circle>
          <circle cx="12" cy="19" r="1"></circle>
        </svg>
      </Button>
      
      <div class="flex items-center gap-1 ml-2">
        <div class="flex -space-x-2">
          <Avatar class="h-8 w-8 border-2 border-white">
            <AvatarFallback class="text-xs">GJC</AvatarFallback>
          </Avatar>
          <Avatar class="h-8 w-8 border-2 border-white">
            <AvatarImage src="" alt="User" />
            <AvatarFallback class="bg-gray-200">
              <span class="sr-only">User</span>
              <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-500">
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