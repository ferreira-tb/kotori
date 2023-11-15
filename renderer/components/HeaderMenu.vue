<script setup lang="ts"></script>

<template>
    <nav class="kt-header-menu" @click="$router.push({ name: 'home' })">
        Home
    </nav>
</template>

<style scoped>
.kt-header-menu {
    display: flex;
    user-select: none;
    align-items: center;
    justify-content: flex-start;
    gap: 8px;
    padding: 1px;
    cursor: pointer;
}
</style>
