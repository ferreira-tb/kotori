<script setup lang="ts">
const router = useRouter();

onKeyDown('F1', () => void router.push({ name: 'library' }));
</script>

<template>
  <m-scaffold default-border="none">
    <template #default>
      <router-view #default="{ Component }">
        <template v-if="Component">
          <component :is="Component" />
        </template>
      </router-view>
    </template>
  </m-scaffold>
</template>
