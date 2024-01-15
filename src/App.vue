<script setup lang="ts">
import { MScaffold } from 'manatsu';
</script>

<template>
  <MScaffold>
    <RouterView #default="{ Component }">
      <template v-if="Component">
        <Transition mode="out-in">
          <KeepAlive>
            <Suspense>
              <component :is="Component" />
              <template #fallback>
                <img
                  src="/icons/loading.svg"
                  alt="loading"
                  class="absolute left-2/4 top-2/4 -translate-y-2/4 translate-x-2/4"
                />
              </template>
            </Suspense>
          </KeepAlive>
        </Transition>
      </template>
    </RouterView>
  </MScaffold>
</template>
