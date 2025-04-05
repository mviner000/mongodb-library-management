<!-- src/components/MongoDBTableNavbar.vue -->
<script setup lang="ts">
import { inject } from 'vue';
import { useRouter } from 'vue-router';
import { Avatar, AvatarImage, AvatarFallback } from '@/components/ui/avatar';

defineProps<{
  title?: string;
}>();

const sidebarState = inject('sidebarState') as {
  isOpen: { value: boolean },
  toggle: () => void,
  close: () => void
}

const router = useRouter();

const menuItems = [
  'File', 'Edit', 'View', 'Insert', 'Format', 'Data', 'Tools', 'Extensions', 'Help'
];
</script>

<template>
  <header class="sticky top-0 z-50 flex items-center w-full h-14 px-2 border-b bg-white">
    <!-- Left section: Menu and title -->
    <div class="flex items-center gap-2">
        
      <Button variant="ghost" size="icon" class="text-gray-500" @click="sidebarState.toggle()">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="12" x2="21" y2="12"></line>
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="3" y1="18" x2="21" y2="18"></line>
        </svg>
      </Button>


      <Button 
          variant="ghost" 
          size="sm" 
          @click="router.push('/home')"
          class="text-gray-500 hover:text-gray-800"
        >
        Home
      </Button>
      
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