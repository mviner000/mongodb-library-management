<!-- src/App.vue -->
<script setup lang="ts">
import LibraryAttendanceRecord from "./components/LibraryAttendanceRecord.vue";
import Toaster from "./components/ui/toast/Toaster.vue";
</script>

<template>
  <Toaster />
  <div class="container mx-auto p-4">
    <LibraryAttendanceRecord />
  </div>
</template>
