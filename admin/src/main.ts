// src/main.ts

import { createApp } from "vue";
import App from "./App.vue";
import router from "./router";
import './assets/index.css';
import { createPinia } from 'pinia'; // Import createPinia

const pinia = createPinia(); // Create the Pinia instance
const app = createApp(App);

app.use(router);
app.use(pinia); // Use Pinia
app.mount("#app");