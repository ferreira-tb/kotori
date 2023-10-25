<script setup lang="ts">
import LibraryMenuSearch from '@/components/LibraryMenuSearch.vue';

const emit = defineEmits<{
	(e: 'search', value: string): void;
}>();
</script>

<template>
	<nav class="flex w-full items-center justify-between p-1">
		<div class="w-full"></div>
		<LibraryMenuSearch @search="(value) => $emit('search', value)" />
	</nav>
</template>
