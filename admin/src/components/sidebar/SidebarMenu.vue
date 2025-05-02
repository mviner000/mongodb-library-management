<!-- src/components/sidebar/SidebarMenu.vue -->
<script setup lang="ts">
import { useRouter } from 'vue-router';
import SettingsNavItem from './SettingsNavItem.vue';

defineProps<{
  isOpen: boolean;
}>();

const router = useRouter()

const goToHome = () => {
  router.replace('/home')
}

const emit = defineEmits(['close']);

const closeMenu = () => {
  emit('close');
};
</script>

<template>
  <div v-if="isOpen" 
       class="fixed inset-0 bg-black bg-opacity-50 z-40"
       @click="closeMenu">
  </div>
  
  <aside 
    :class="[
      'fixed left-0 top-0 bottom-0 z-50 w-64 bg-white transform transition-transform duration-300 ease-in-out',
      isOpen ? 'translate-x-0' : '-translate-x-full'
    ]"
  >
    <div class="flex items-center gap-4 h-14 px-4 border-b">
      <button @click="closeMenu" class="text-gray-500">
        <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="3" y1="12" x2="21" y2="12"></line>
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="3" y1="18" x2="21" y2="18"></line>
        </svg>
      </button>
      <div @click="goToHome" class="flex items-center cursor-pointer">
        <div class="w-6 h-6">
          <svg viewBox="0 0 24 24" width="24" height="24">
            <rect fill="#0F9D58" x="0" y="0" width="24" height="24" rx="4" />
            <rect fill="#FFFFFF" x="4" y="4" width="16" height="16" rx="2" />
            <rect fill="#0F9D58" x="6" y="6" width="12" height="2" />
            <rect fill="#0F9D58" x="6" y="10" width="12" height="2" />
            <rect fill="#0F9D58" x="6" y="14" width="12" height="2" />
          </svg>
        </div>
        <span class="ml-1 font-bold text-lg text-black">Damavz</span>
        <span class="text-xs ml-1 text-gray-500">PH</span>
      </div>
    </div>

    <nav class="overflow-y-auto h-full">
      <div class="px-3 py-2">
        <h3 class="font-bold text-base px-3 py-1">Explore</h3>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600">
            <path d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"></path>
          </svg>
          <span class="text-sm">Trending</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600">
            <path d="M9 18V5l12-2v13"></path>
            <circle cx="6" cy="18" r="3"></circle>
            <circle cx="18" cy="16" r="3"></circle>
          </svg>
          <span class="text-sm">Music</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600">
            <rect x="2" y="2" width="20" height="20" rx="2.18" ry="2.18"></rect>
            <line x1="7" y1="2" x2="7" y2="22"></line>
            <line x1="17" y1="2" x2="17" y2="22"></line>
            <line x1="2" y1="12" x2="22" y2="12"></line>
          </svg>
          <span class="text-sm">Movies</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600">
            <polygon points="5 3 19 12 5 21 5 3"></polygon>
          </svg>
          <span class="text-sm">Gaming</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600">
            <path d="M4 22h16a2 2 0 0 0 2-2V4a2 2 0 0 0-2-2H8a2 2 0 0 0-2 2v16a2 2 0 0 1-2 2Zm0 0a2 2 0 0 1-2-2v-9c0-1.1.9-2 2-2h2"></path>
          </svg>
          <span class="text-sm">News</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600">
            <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
          </svg>
          <span class="text-sm">Sports</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600">
            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78L12 21.23l8.84-8.84a5.5 5.5 0 0 0 0-7.78z"></path>
          </svg>
          <span class="text-sm">Fashion & Beauty</span>
        </a>
      </div>
      
      <div class="border-t my-2"></div>
      
      <div class="px-3 py-2">
        <h3 class="font-bold text-base px-3 py-1">More from Damavz</h3>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="green" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
            <circle cx="12" cy="12" r="4"></circle>
            <path d="M12 12v.01"></path>
          </svg>
          <span class="text-sm">Damavz Premium</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="green" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M12 20h9"></path>
            <path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"></path>
          </svg>
          <span class="text-sm">Damavz Studio</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="green" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
          <span class="text-sm">Damavz Music</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="green" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
          </svg>
          <span class="text-sm">Damavz Kids</span>
        </a>
      </div>
      
      <div class="border-t my-2"></div>
      
      <div class="px-3 py-2">
        <SettingsNavItem />
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600">
            <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
            <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
          </svg>
          <span class="text-sm">Report history</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600">
            <circle cx="12" cy="12" r="10"></circle>
            <path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"></path>
            <line x1="12" y1="17" x2="12.01" y2="17"></line>
          </svg>
          <span class="text-sm">Help</span>
        </a>
        
        <a href="#" class="flex items-center gap-6 px-3 py-2 hover:bg-gray-100 rounded-lg">
          <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="text-gray-600">
            <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path>
          </svg>
          <span class="text-sm">Send feedback</span>
        </a>
      </div>
      
      <div class="border-t my-2"></div>
      
      <div class="px-6 py-4 text-xs text-gray-500">
        <div class="flex flex-wrap gap-x-3 gap-y-2">
          <a href="#" class="hover:underline">About</a>
          <a href="#" class="hover:underline">Press</a>
          <a href="#" class="hover:underline">Copyright</a>
          <a href="#" class="hover:underline">Contact us</a>
          <a href="#" class="hover:underline">Creators</a>
          <a href="#" class="hover:underline">Advertise</a>
          <a href="#" class="hover:underline">Developers</a>
        </div>
        
        <div class="flex flex-wrap gap-x-3 gap-y-2 mt-4">
          <a href="#" class="hover:underline">Terms</a>
          <a href="#" class="hover:underline">Privacy</a>
          <a href="#" class="hover:underline">Policy & Safety</a>
          <a href="#" class="hover:underline">How Damavz works</a>
          <a href="#" class="hover:underline">Test new features</a>
        </div>
        
        <div class="my-4 mb-14">
          © 2025 OurCompany LLC
        </div>
      </div>
    </nav>
  </aside>
</template>