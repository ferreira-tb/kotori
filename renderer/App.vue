<script setup lang="ts">
import { RouterView } from 'vue-router';
import HeaderMenu from '@/components/HeaderMenu.vue';
</script>

<template>
    <main class="kt-main">
        <header class="kt-main-header">
            <HeaderMenu />
        </header>
        <div class="kt-main-content">
            <RouterView #default="{ Component }">
                <template v-if="Component">
                    <Transition name="fade" mode="out-in">
                        <Suspense>
                            <component :is="Component" />
                        </Suspense>
                    </Transition>
                </template>
            </RouterView>
        </div>
    </main>
</template>

<style scoped>
.kt-main {
    --kt-header-height: 2rem;

    height: 100vh;
    width: 100vw;
    display: flex;
    flex-direction: column;
}

.kt-main-header {
    height: var(--kt-header-height);
    flex-shrink: 0;
    flex-grow: 0;
}

.kt-main-content {
    width: 100%;
    height: calc(100% - var(--kt-header-height));
    overflow: hidden;
}
</style>
