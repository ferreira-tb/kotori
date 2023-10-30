import { VisualNovel, VisualNovelImage } from '@/database';
import { readonly, ref, shallowRef } from 'vue';
import { RendererProcessError } from '@/utils/error';

function cache() {
	const novels = shallowRef<VisualNovel[]>([]);
	const isLoading = ref(false);

	async function reload() {
		try {
			isLoading.value = true;
			novels.value = await VisualNovel.findAll({ include: VisualNovelImage });
		} catch (err) {
			RendererProcessError.catch(err);
		} finally {
			isLoading.value = false;
		}
	}

	return function () {
		if (novels.value.length === 0 && !isLoading.value) {
			reload();
		}

		return {
			novels,
			isLoading: readonly(isLoading),
			reload
		};
	};
}

export const useVisualNovels = cache();
