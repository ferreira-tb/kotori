<script setup lang="ts">
import LibraryGrid from '@/components/LibraryGrid.vue';
import LibraryMenu from '@/components/LibraryMenu.vue';
import LibraryMessageEmpty from '@/components/LibraryMessageEmpty.vue';
import { useVisualNovels } from '@/composables';
import { VisualNovel, VisualNovelImage } from '@/database';
import { VNDB } from 'vndb-query';
import { triggerRef } from 'vue';
import { z } from 'zod';

const vndb = new VNDB();
const { novels } = useVisualNovels();

const schema = z.object({
	id: z.string().regex(VNDB.regex.id.vn),
	title: z.string(),
	alttitle: z.string(),
	olang: z.string(),
	devstatus: z.union([z.literal(0), z.literal(1), z.literal(2)]),
	image: z.object({
		id: z.string(),
		url: z.string().url(),
		dims: z.tuple([z.number().int(), z.number().int()]),
		sexual: z.number().min(0).max(2),
		violence: z.number().min(0).max(2),
		votecount: z.number().int()
	})
});

async function fetchWorks(title: string) {
	let { results } = await vndb.search('vn', title, {
		results: 50,
		fields: [
			'title',
			'alttitle',
			'olang',
			'devstatus',
			'image.id',
			'image.url',
			'image.dims',
			'image.sexual',
			'image.violence',
			'image.votecount'
		]
	});

	results = results.filter((value) => {
		return novels.value.every(({ id }) => id !== value.id.trim());
	});

	const models = await VisualNovel.sequelize.transaction(async () => {
		return await Promise.all(
			results.map((value) => {
				return VisualNovel.sequelize.transaction(async () => {
					const vn = schema.parse(value);
					return await VisualNovel.create(vn, {
						include: VisualNovelImage
					});
				});
			})
		);
	});

	novels.value.push(...models);
	triggerRef(novels);
}
</script>

<template>
	<div class="kt-library">
		<LibraryMenu class="kt-library-header" @search="fetchWorks" />
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
