<script setup lang="ts">
import { MNavbar, MScaffold } from 'manatsu';
</script>

<template>
  <MScaffold>
    <template #header>
      <MNavbar title="Kotori" />
    </template>

    <RouterView #default="{ Component }">
      <template v-if="Component">
        <Transition mode="out-in">
          <KeepAlive>
            <Suspense>
              <component :is="Component" />
              <template #fallback>Loading...</template>
            </Suspense>
          </KeepAlive>
        </Transition>
      </template>
    </RouterView>
  </MScaffold>
</template>
