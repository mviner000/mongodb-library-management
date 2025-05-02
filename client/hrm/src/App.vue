<!-- src/App.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const greetingMessage = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetingMessage.value = await invoke("greet", { name: name.value });
}

onMounted(() => {
  // Set a default name
  name.value = "Tauri";
  greet();
});
</script>

<template>
  <div class="container mx-auto p-4">
    <h1 class="text-2xl font-bold mb-6">Tauri + Vue</h1>

    <div class="flex flex-col items-center">
      <div class="bg-gray-100 p-6 rounded-lg shadow-md w-full max-w-md">
        <h2 class="text-xl font-semibold mb-4">Greeting</h2>

        <form class="flex flex-col" @submit.prevent="greet">
          <input
            id="greet-input"
            v-model="name"
            placeholder="Enter a name..."
            class="px-4 py-2 mb-4 border rounded"
          />
          <button
            type="submit"
            class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
          >
            Greet
          </button>
        </form>

        <p class="mt-4">{{ greetingMessage }}</p>
      </div>
    </div>
  </div>
</template>
