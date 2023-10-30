<script setup lang="ts">
import LibraryGrid from '@/components/LibraryGrid.vue';
import LibraryMenu from '@/components/LibraryMenu.vue';
import LibraryMessageEmpty from '@/components/LibraryMessageEmpty.vue';
import { useVisualNovels } from '@/composables';

const { novels } = useVisualNovels();
</script>

<template>
	<div class="kt-library">
		<LibraryMenu class="kt-library-header" />
		<div class="kt-library-content">
			<LibraryMessageEmpty v-if="novels.length === 0" />
			<LibraryGrid v-else :novels="novels" />
		</div>
	</div>
</template>

<style scoped>
.kt-library {
	width: 100%;
	height: 100%;
	display: flex;
	flex-direction: column;
	overflow: hidden;
}

.kt-library-header {
	height: 2rem;
}

.kt-library-content {
	width: 100%;
	height: calc(100% - 2rem);
	display: flex;
	align-items: center;
	justify-content: center;
}
</style>
