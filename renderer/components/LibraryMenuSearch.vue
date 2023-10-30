<script setup lang="ts">
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

async function search(e: Event) {
	const title = (e.currentTarget as HTMLInputElement | null)?.value;
	if (typeof title !== 'string' || title.length === 0) return;

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
	<div>
		<input type="text" placeholder="Search" @change="search" />
	</div>
</template>
