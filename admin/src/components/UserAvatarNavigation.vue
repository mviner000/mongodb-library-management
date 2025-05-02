<!-- src/components/UserAvatarNavigation.vue -->
<script setup lang="ts">
import { Avatar, AvatarImage, AvatarFallback } from '@/components/ui/avatar';
import { Button } from '@/components/ui/button';

// You can add props for customization if needed
defineProps<{
  // Add props as needed, for example:
  // users?: Array<{ initials?: string, image?: string }>;
}>();
</script>

<template>
  <div class="flex items-center gap-2">
    <!-- More options button -->
    <Button variant="ghost" size="icon" class="text-gray-500">
      <!-- More Vertical Icon (3 dots) -->
      <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="12" cy="12" r="1"></circle>
        <circle cx="12" cy="5" r="1"></circle>
        <circle cx="12" cy="19" r="1"></circle>
      </svg>
    </Button>
    
    <!-- Avatar group -->
    <div class="flex items-center gap-1 ml-2">
      <div class="flex -space-x-2">
        <!-- First avatar with initials -->
        <Avatar class="h-8 w-8 border-2 border-white">
          <AvatarFallback class="text-xs">GJC</AvatarFallback>
        </Avatar>
        
        <!-- Second avatar with image fallback -->
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
</template>