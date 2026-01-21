<script setup lang="ts">
import TopNavbar from './TopNavbar.vue'
import ProjectFormDialog from '@/components/projects/ProjectFormDialog.vue'
import TaskFormDialog from '@/components/tasks/TaskFormDialog.vue'
</script>

<template>
  <div class="min-h-screen bg-bg-primary">
    <TopNavbar />
    <main class="pt-navbar">
      <div class="max-w-7xl mx-auto px-4 py-6">
        <slot />
      </div>
    </main>

    <!-- Global Dialogs -->
    <ProjectFormDialog />
    <TaskFormDialog />
  </div>
</template>
