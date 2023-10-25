<script setup lang="ts">
import { shallowRef, triggerRef } from 'vue';
import { VNDB, type ResponsePostVisualNovel } from 'vndb-query';
import LibraryMenu from '@/components/LibraryMenu.vue';
import LibraryGrid from '@/components/LibraryGrid.vue';
import LibraryMessageEmpty from '@/components/LibraryMessageEmpty.vue';

const vndb = new VNDB();
const works = shallowRef<ResponsePostVisualNovel[]>([]);

async function fetchWorks(title: string) {
	const { results } = await vndb.search('vn', title, {
		fields: ['title', 'alttitle', 'image.url'],
		results: 10
	});

	works.value.push(...results);
	triggerRef(works);
}
</script>

<template>
	<div class="flex h-full w-full flex-col">
		<LibraryMenu @search="fetchWorks" />
		<div class="flex h-full w-full items-center justify-center">
			<LibraryMessageEmpty v-if="works.length === 0" />
			<LibraryGrid v-else :works="works" />
		</div>
	</div>
</template>
