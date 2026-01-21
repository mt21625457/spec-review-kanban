/**
 * Projects Store - Client state for project management
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Project } from '@/types'

export const useProjectsStore = defineStore('projects', () => {
  // State
  const currentProjectId = ref<string | null>(null)
  const projects = ref<Project[]>([])

  // Getters
  const currentProject = computed(() =>
    projects.value.find((p) => p.id === currentProjectId.value)
  )

  const hasProjects = computed(() => projects.value.length > 0)

  // Actions
  function setCurrentProject(projectId: string | null) {
    currentProjectId.value = projectId
    // Persist to localStorage
    if (projectId) {
      localStorage.setItem('currentProjectId', projectId)
    } else {
      localStorage.removeItem('currentProjectId')
    }
  }

  function setProjects(newProjects: Project[]) {
    projects.value = newProjects
    // Auto-select first project if none selected
    if (!currentProjectId.value && newProjects.length > 0) {
      // Try to restore from localStorage first
      const savedId = localStorage.getItem('currentProjectId')
      if (savedId && newProjects.some((p) => p.id === savedId)) {
        currentProjectId.value = savedId
      } else {
        currentProjectId.value = newProjects[0].id
      }
    }
  }

  function addProject(project: Project) {
    projects.value.push(project)
    if (!currentProjectId.value) {
      setCurrentProject(project.id)
    }
  }

  function updateProject(projectId: string, updates: Partial<Project>) {
    const index = projects.value.findIndex((p) => p.id === projectId)
    if (index !== -1) {
      projects.value[index] = { ...projects.value[index], ...updates }
    }
  }

  function removeProject(projectId: string) {
    projects.value = projects.value.filter((p) => p.id !== projectId)
    if (currentProjectId.value === projectId) {
      setCurrentProject(projects.value[0]?.id ?? null)
    }
  }

  // Initialize from localStorage
  function init() {
    const savedId = localStorage.getItem('currentProjectId')
    if (savedId) {
      currentProjectId.value = savedId
    }
  }

  return {
    // State
    currentProjectId,
    projects,
    // Getters
    currentProject,
    hasProjects,
    // Actions
    setCurrentProject,
    setProjects,
    addProject,
    updateProject,
    removeProject,
    init,
  }
})
