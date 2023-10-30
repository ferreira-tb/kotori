<script setup lang="ts">
import { computed } from 'vue';
import type { VisualNovel } from '@/database';

const props = defineProps<{
	novel: VisualNovel;
}>();

const url = computed(() => `url("${props.novel.image!.url}")`);
</script>

<template>
	<div class="kt-library-grid-item"></div>
</template>

<style scoped>
.kt-library-grid-item {
	width: 11rem;
	height: 14rem;
	overflow: hidden;
	background-size: cover;
	background-repeat: no-repeat;
	background-image: v-bind('url');
	background-position: center;
	border-radius: 0.25rem;
	cursor: pointer;
}

.kt-library-grid-item:hover {
	filter: brightness(0.8);
	transition: filter 0.2s ease-in-out;
}
</style>
