// src/router/index.ts

import { createRouter, createWebHistory } from 'vue-router'
import TemplateGallery from '@/components/TemplateGallery.vue'
import MongoDBDataTable from '@/components/MongoDBDataTable.vue'
import HelloWorldTab from '@/components/HelloWorldTab.vue'

const routes = [
  {
    path: '/',
    redirect: '/home'
  },
  {
    path: '/home',
    name: 'Home',
    component: TemplateGallery
  },
  {
    path: '/collection/:name',
    name: 'Collection',
    component: MongoDBDataTable,
    props: true
  },
  {
    path: '/hello',
    name: 'Hello',
    component: HelloWorldTab
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/components/MongoDBStatus.vue')
  }
]

const router = createRouter({
  history: createWebHistory('/app'), // Set base path to /app
  routes
})

export default router