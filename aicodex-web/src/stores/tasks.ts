import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface Task {
  id: string
  title: string
  description?: string
  status: 'todo' | 'doing' | 'review' | 'done'
  projectId: string
  order: number
  createdAt: string
  updatedAt: string
}

export interface Project {
  id: string
  name: string
  description?: string
}

export const useTasksStore = defineStore('tasks', () => {
  const projects = ref<Project[]>([])
  const tasks = ref<Task[]>([])
  const currentProjectId = ref<string | null>(null)
  const isLoading = ref(false)

  const setProjects = (newProjects: Project[]) => {
    projects.value = newProjects
  }

  const setTasks = (newTasks: Task[]) => {
    tasks.value = newTasks
  }

  const setCurrentProject = (projectId: string | null) => {
    currentProjectId.value = projectId
  }

  const updateTaskStatus = (taskId: string, status: Task['status']) => {
    const task = tasks.value.find((t) => t.id === taskId)
    if (task) {
      task.status = status
      task.updatedAt = new Date().toISOString()
    }
  }

  const updateTaskOrder = (taskId: string, newOrder: number) => {
    const task = tasks.value.find((t) => t.id === taskId)
    if (task) {
      task.order = newOrder
    }
  }

  const addTask = (task: Task) => {
    tasks.value.push(task)
  }

  const removeTask = (taskId: string) => {
    const index = tasks.value.findIndex((t) => t.id === taskId)
    if (index !== -1) {
      tasks.value.splice(index, 1)
    }
  }

  const getTasksByStatus = (status: Task['status']) => {
    return tasks.value
      .filter((t) => t.status === status && t.projectId === currentProjectId.value)
      .sort((a, b) => a.order - b.order)
  }

  return {
    projects,
    tasks,
    currentProjectId,
    isLoading,
    setProjects,
    setTasks,
    setCurrentProject,
    updateTaskStatus,
    updateTaskOrder,
    addTask,
    removeTask,
    getTasksByStatus,
  }
})
