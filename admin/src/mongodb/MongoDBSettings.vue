<!-- src/components/mongodb/MongoDBSettings.vue -->
<script setup lang="ts">
import HomeNavbar from '@/components/HomeNavbar.vue';
import LibraryInitializer from './LibraryInitializer.vue';
import MongoDBStatus from './MongoDBStatus.vue';
</script>

<template>
    <HomeNavbar :title="'Home'" class="sticky top-0 z-50"/>
     <div class="bg-gray-50 py-6 px-4">
        <div class="max-w-7xl mx-auto">

            <div class="space-y-6">
                <h1 class="text-2xl font-bold my-12 text-center">
                    MongoDB Settings
                </h1>
                
                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                    <!-- MongoDB Status Panel -->
                    <div class="lg:col-span-2">
                    <MongoDBStatus />
                    </div>
                    
                    <!-- Collections Panels - 2 columns on lg screens -->
                    <div class="col-span-1">
                    <LibraryInitializer />
                    </div>

                    <div class="col-span-1">
                    <LibraryInitializer />
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>