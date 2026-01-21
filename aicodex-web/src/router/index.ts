import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { getAuthToken } from '@/lib/api'

const routes: RouteRecordRaw[] = [
  // 公开路由
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/pages/LoginPage.vue'),
    meta: { title: 'auth.login.title', public: true }
  },
  // 需要认证的路由
  {
    path: '/',
    redirect: '/dashboard'
  },
  {
    path: '/dashboard',
    name: 'Dashboard',
    component: () => import('@/pages/Dashboard.vue'),
    meta: { title: 'nav.dashboard', requiresAuth: true }
  },
  {
    path: '/reviews',
    name: 'Reviews',
    component: () => import('@/pages/ReviewList.vue'),
    meta: { title: 'nav.reviews', requiresAuth: true }
  },
  {
    path: '/reviews/:id',
    name: 'ReviewDetail',
    component: () => import('@/pages/ReviewDetail.vue'),
    meta: { title: 'review.detail', requiresAuth: true }
  },
  {
    path: '/tasks',
    name: 'Tasks',
    component: () => import('@/pages/Tasks.vue'),
    meta: { title: 'nav.tasks', requiresAuth: true }
  },
  {
    path: '/tasks/:projectId',
    name: 'ProjectTasks',
    component: () => import('@/pages/ProjectTasks.vue'),
    meta: { title: 'nav.tasks', requiresAuth: true }
  },
  {
    path: '/settings',
    component: () => import('@/pages/settings/SettingsLayout.vue'),
    meta: { title: 'nav.settings', requiresAuth: true },
    children: [
      {
        path: '',
        redirect: '/settings/general'
      },
      {
        path: 'general',
        name: 'SettingsGeneral',
        component: () => import('@/pages/settings/GeneralSettings.vue'),
        meta: { title: 'settings.nav.general' }
      },
      {
        path: 'projects',
        name: 'SettingsProjects',
        component: () => import('@/pages/settings/ProjectSettings.vue'),
        meta: { title: 'settings.nav.projects' }
      },
      {
        path: 'repos',
        name: 'SettingsRepos',
        component: () => import('@/pages/settings/RepoSettings.vue'),
        meta: { title: 'settings.nav.repos' }
      },
      {
        path: 'agents',
        name: 'SettingsAgents',
        component: () => import('@/pages/settings/AgentSettings.vue'),
        meta: { title: 'settings.nav.agents' }
      },
      {
        path: 'mcp',
        name: 'SettingsMcp',
        component: () => import('@/pages/settings/McpSettings.vue'),
        meta: { title: 'settings.nav.mcp' }
      }
    ]
  },
  {
    path: '/repo-mappings',
    name: 'RepoMappings',
    component: () => import('@/pages/RepoMappings.vue'),
    meta: { title: 'settings.repoMappings', requiresAuth: true }
  },
  // 管理员路由
  {
    path: '/admin',
    redirect: '/admin/users'
  },
  {
    path: '/admin/users',
    name: 'AdminUsers',
    component: () => import('@/pages/admin/UsersPage.vue'),
    meta: { title: 'admin.users.title', requiresAuth: true, requiresAdmin: true }
  },
  {
    path: '/admin/instances',
    name: 'AdminInstances',
    component: () => import('@/pages/admin/InstancesPage.vue'),
    meta: { title: 'admin.instances.title', requiresAuth: true, requiresAdmin: true }
  },
  {
    path: '/admin/instances/:id',
    name: 'AdminInstanceDetail',
    component: () => import('@/pages/admin/InstanceDetailPage.vue'),
    meta: { title: 'admin.instances.detail', requiresAuth: true, requiresAdmin: true }
  },
  // 兼容旧路由
  {
    path: '/vibe',
    redirect: '/tasks'
  },
  {
    path: '/vibe/:pathMatch(.*)*',
    redirect: to => ({ path: `/tasks${to.params.pathMatch ? '/' + (to.params.pathMatch as string[]).join('/') : ''}` })
  },
  // 404
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: () => import('@/pages/NotFound.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// 路由守卫 - 认证检查和页面标题
router.beforeEach((to, _from, next) => {
  // 更新页面标题
  document.title = `${to.meta.title ? to.meta.title + ' - ' : ''}多智能体编排系统`

  // 检查是否需要认证
  const requiresAuth = to.matched.some(record => record.meta.requiresAuth)
  const requiresAdmin = to.matched.some(record => record.meta.requiresAdmin)
  const isPublicRoute = to.matched.some(record => record.meta.public)
  const token = getAuthToken()
  const authStore = useAuthStore()

  // 已登录用户访问登录页，跳转到首页
  if (isPublicRoute && token) {
    next({ path: '/dashboard' })
    return
  }

  // 需要认证但未登录，跳转到登录页
  if (requiresAuth && !token) {
    next({
      path: '/login',
      query: { redirect: to.fullPath }
    })
    return
  }

  // 需要管理员权限但用户不是管理员
  if (requiresAdmin && authStore.user && authStore.user.role !== 'admin') {
    next({ path: '/dashboard' })
    return
  }

  next()
})

// 路由守卫 - 验证 token 有效性
router.afterEach(async (to) => {
  const authStore = useAuthStore()
  const token = getAuthToken()

  // 如果有 token 但 store 未初始化，尝试获取用户信息
  if (token && !authStore.isInitialized && !to.meta.public) {
    // 用户信息将由 useAuthInit 在 App.vue 中加载
  }
})

export default router
