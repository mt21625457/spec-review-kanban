<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { Button } from '@/components/ui'

const { t } = useI18n()
</script>

<template>
  <div class="min-h-[60vh] flex flex-col items-center justify-center">
    <div class="text-center">
      <h1 class="text-9xl font-bold text-text-muted">404</h1>
      <h2 class="mt-4 text-2xl font-semibold text-text-primary">
        {{ t('common.notFound') }}
      </h2>
      <p class="mt-2 text-text-muted">
        {{ t('common.notFoundDesc') }}
      </p>
      <div class="mt-8">
        <router-link to="/">
          <Button variant="primary">
            {{ t('common.backHome') }}
          </Button>
        </router-link>
      </div>
    </div>
  </div>
</template>
