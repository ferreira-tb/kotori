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
