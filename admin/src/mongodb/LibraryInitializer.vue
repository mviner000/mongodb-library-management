<!-- src/components/mongodb/LibraryInitializer.vue -->
<template>
    <div class="border rounded-md p-4 w-full">
    <h2 class="text-xl font-bold mb-4">Library Collections</h2>
    
      <p class="text-sm text-gray-600 mb-4">
        Initialize default library collections with predefined schemas for books, authors, genres, and more.
      </p>
      
      <div class="flex justify-between items-center">
        <button 
          @click="openConfirmDialog" 
          class="px-4 py-2 bg-primary text-white rounded-md hover:bg-primary/90"
          :disabled="isLoading"
        >
          <span v-if="isLoading" class="flex items-center">
            <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
            Initializing...
          </span>
          <span v-else>Initialize Library Collections</span>
        </button>
      </div>
  
      <!-- Confirmation Dialog -->
      <AlertDialog :open="isDialogOpen" @update:open="isDialogOpen = $event">
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Initialize Library Collections</AlertDialogTitle>
            <AlertDialogDescription>
              This will create predefined collections for books, authors, genres, and publishers with proper schemas. 
              Existing collections will not be affected.
              <br><br>
              Are you sure you want to proceed?
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel @click="isDialogOpen = false">Cancel</AlertDialogCancel>
            <AlertDialogAction @click="initializeLibraryCollections">Initialize</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  </template>
  
<script setup>
  import { ref } from 'vue';
  import { invoke } from '@tauri-apps/api/core'
  import { useToast } from '@/components/ui/toast';
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '@/components/ui/alert-dialog';
  
  const { toast } = useToast();
  const isDialogOpen = ref(false);
  const isLoading = ref(false);
  
  function openConfirmDialog() {
    isDialogOpen.value = true;
  }
  
  async function initializeLibraryCollections() {
    isDialogOpen.value = false;
    isLoading.value = true;
    
    try {
      // First try to initialize via local Tauri API
      await invoke('initialize_library_collections');
      toast({
        title: "Success!",
        description: "Library collections initialized successfully.",
        variant: "success"
      });
    } catch (error) {
      // If local API fails, try the REST API
      try {
        const response = await fetch('http://localhost:3000/api/initialize-library-collections', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          }
        });
        
        const data = await response.json();
        
        if (data.success) {
          toast({
            title: "Success!",
            description: "Library collections initialized successfully.",
            variant: "success"
          });
        } else {
          throw new Error(data.error || "Unknown error");
        }
      } catch (apiError) {
        toast({
          title: "Error",
          description: `Failed to initialize library collections: ${apiError.message}`,
          variant: "destructive"
        });
      }
    } finally {
      isLoading.value = false;
    }
  }
  </script>