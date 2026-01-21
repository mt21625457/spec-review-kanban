/**
 * useProjects - Vue Query hooks for project management
 */
import { computed, type Ref } from 'vue'
import { useQuery, useMutation, useQueryClient } from '@tanstack/vue-query'
import { projectsApi } from '@/lib/api'
import { queryKeys } from '@/lib/queryClient'
import type { Project, CreateProject, UpdateProject, Repo, CreateProjectRepo } from '@/types'

/**
 * Fetch all projects
 */
export function useProjects() {
  return useQuery({
    queryKey: queryKeys.projects.all,
    queryFn: () => projectsApi.list(),
    staleTime: 60 * 1000, // 1 minute
  })
}

/**
 * Fetch a single project by ID
 */
export function useProject(projectId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      projectId.value ? queryKeys.projects.detail(projectId.value) : ['projects', null]
    ),
    queryFn: () => {
      if (!projectId.value) throw new Error('Project ID is required')
      return projectsApi.get(projectId.value)
    },
    enabled: computed(() => !!projectId.value),
  })
}

/**
 * Fetch project repositories
 */
export function useProjectRepositories(projectId: Ref<string | null>) {
  return useQuery({
    queryKey: computed(() =>
      projectId.value ? queryKeys.projects.repositories(projectId.value) : ['projects', null, 'repositories']
    ),
    queryFn: () => {
      if (!projectId.value) throw new Error('Project ID is required')
      return projectsApi.getRepositories(projectId.value)
    },
    enabled: computed(() => !!projectId.value),
  })
}

/**
 * Create a new project
 */
export function useCreateProject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (data: CreateProject) => projectsApi.create(data),
    onSuccess: (newProject) => {
      // Invalidate and refetch projects list
      queryClient.invalidateQueries({ queryKey: queryKeys.projects.all })
      // Optimistically add the new project to cache
      queryClient.setQueryData<Project[]>(queryKeys.projects.all, (old) =>
        old ? [...old, newProject] : [newProject]
      )
    },
  })
}

/**
 * Update an existing project
 */
export function useUpdateProject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, data }: { id: string; data: UpdateProject }) =>
      projectsApi.update(id, data),
    onSuccess: (updatedProject, { id }) => {
      // Update the project in cache
      queryClient.setQueryData(queryKeys.projects.detail(id), updatedProject)
      // Update in the list cache
      queryClient.setQueryData<Project[]>(queryKeys.projects.all, (old) =>
        old?.map((p) => (p.id === id ? updatedProject : p))
      )
    },
  })
}

/**
 * Delete a project
 */
export function useDeleteProject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) => projectsApi.delete(id),
    onSuccess: (_, id) => {
      // Remove from cache
      queryClient.removeQueries({ queryKey: queryKeys.projects.detail(id) })
      // Remove from list cache
      queryClient.setQueryData<Project[]>(queryKeys.projects.all, (old) =>
        old?.filter((p) => p.id !== id)
      )
    },
  })
}

/**
 * Add repository to project
 */
export function useAddProjectRepository() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ projectId, data }: { projectId: string; data: CreateProjectRepo }) =>
      projectsApi.addRepository(projectId, data),
    onSuccess: (newRepo, { projectId }) => {
      // Invalidate repositories cache
      queryClient.invalidateQueries({ queryKey: queryKeys.projects.repositories(projectId) })
      // Optimistically add to cache
      queryClient.setQueryData<Repo[]>(queryKeys.projects.repositories(projectId), (old) =>
        old ? [...old, newRepo] : [newRepo]
      )
    },
  })
}

/**
 * Remove repository from project
 */
export function useDeleteProjectRepository() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ projectId, repoId }: { projectId: string; repoId: string }) =>
      projectsApi.deleteRepository(projectId, repoId),
    onSuccess: (_, { projectId, repoId }) => {
      // Remove from cache
      queryClient.setQueryData<Repo[]>(queryKeys.projects.repositories(projectId), (old) =>
        old?.filter((r) => r.id !== repoId)
      )
    },
  })
}
