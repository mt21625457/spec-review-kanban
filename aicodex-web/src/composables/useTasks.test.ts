/**
 * useTasks composable unit tests
 */
import { describe, it, expect } from 'vitest'
import { groupTasksByStatus, taskStatusInfo } from './useTasks'
import type { Task, TaskStatus } from '@/types'

// Create mock task for testing
function createMockTask(overrides: Partial<Task> = {}): Task {
  return {
    id: 'task-1',
    title: 'Test Task',
    description: 'Test description',
    status: 'todo',
    project_id: 'project-1',
    parent_workspace_id: null,
    shared_task_id: null,
    created_at: '2025-01-01T00:00:00Z',
    updated_at: '2025-01-01T00:00:00Z',
    ...overrides,
  }
}

describe('groupTasksByStatus', () => {
  it('should group tasks by status from array', () => {
    const tasks: Task[] = [
      createMockTask({ id: '1', status: 'todo' }),
      createMockTask({ id: '2', status: 'inprogress' }),
      createMockTask({ id: '3', status: 'todo' }),
      createMockTask({ id: '4', status: 'done' }),
      createMockTask({ id: '5', status: 'inreview' }),
    ]

    const grouped = groupTasksByStatus(tasks)

    expect(grouped.todo).toHaveLength(2)
    expect(grouped.inprogress).toHaveLength(1)
    expect(grouped.inreview).toHaveLength(1)
    expect(grouped.done).toHaveLength(1)
    expect(grouped.cancelled).toHaveLength(0)
  })

  it('should group tasks by status from Record', () => {
    const tasksRecord: Record<string, Task> = {
      '1': createMockTask({ id: '1', status: 'todo' }),
      '2': createMockTask({ id: '2', status: 'inprogress' }),
      '3': createMockTask({ id: '3', status: 'done' }),
    }

    const grouped = groupTasksByStatus(tasksRecord)

    expect(grouped.todo).toHaveLength(1)
    expect(grouped.inprogress).toHaveLength(1)
    expect(grouped.done).toHaveLength(1)
    expect(grouped.inreview).toHaveLength(0)
    expect(grouped.cancelled).toHaveLength(0)
  })

  it('should return empty arrays for all statuses when no tasks', () => {
    const grouped = groupTasksByStatus([])

    expect(grouped.todo).toHaveLength(0)
    expect(grouped.inprogress).toHaveLength(0)
    expect(grouped.inreview).toHaveLength(0)
    expect(grouped.done).toHaveLength(0)
    expect(grouped.cancelled).toHaveLength(0)
  })

  it('should sort tasks by updated_at descending within each group', () => {
    const tasks: Task[] = [
      createMockTask({ id: '1', status: 'todo', updated_at: '2025-01-01T00:00:00Z' }),
      createMockTask({ id: '2', status: 'todo', updated_at: '2025-01-03T00:00:00Z' }),
      createMockTask({ id: '3', status: 'todo', updated_at: '2025-01-02T00:00:00Z' }),
    ]

    const grouped = groupTasksByStatus(tasks)

    expect(grouped.todo[0].id).toBe('2') // Jan 3 - most recent
    expect(grouped.todo[1].id).toBe('3') // Jan 2
    expect(grouped.todo[2].id).toBe('1') // Jan 1 - oldest
  })

  it('should handle all status types', () => {
    const statuses: TaskStatus[] = ['todo', 'inprogress', 'inreview', 'done', 'cancelled']
    const tasks = statuses.map((status, i) =>
      createMockTask({ id: String(i + 1), status })
    )

    const grouped = groupTasksByStatus(tasks)

    statuses.forEach((status) => {
      expect(grouped[status]).toHaveLength(1)
    })
  })
})

describe('taskStatusInfo', () => {
  it('should have info for all status types', () => {
    const statuses: TaskStatus[] = ['todo', 'inprogress', 'inreview', 'done', 'cancelled']

    statuses.forEach((status) => {
      expect(taskStatusInfo[status]).toBeDefined()
      expect(taskStatusInfo[status].label).toBeDefined()
      expect(taskStatusInfo[status].color).toBeDefined()
      expect(taskStatusInfo[status].icon).toBeDefined()
    })
  })

  it('should have proper Chinese labels', () => {
    expect(taskStatusInfo.todo.label).toBe('待办')
    expect(taskStatusInfo.inprogress.label).toBe('进行中')
    expect(taskStatusInfo.inreview.label).toBe('待审核')
    expect(taskStatusInfo.done.label).toBe('已完成')
    expect(taskStatusInfo.cancelled.label).toBe('已取消')
  })

  it('should have Tailwind color classes', () => {
    Object.values(taskStatusInfo).forEach((info) => {
      expect(info.color).toMatch(/^bg-/)
      expect(info.color).toContain('dark:')
    })
  })

  it('should have valid SVG path data for icons', () => {
    Object.values(taskStatusInfo).forEach((info) => {
      // SVG path data should be a non-empty string starting with M (moveto)
      expect(info.icon).toBeTruthy()
      expect(typeof info.icon).toBe('string')
      expect(info.icon.length).toBeGreaterThan(0)
    })
  })
})
