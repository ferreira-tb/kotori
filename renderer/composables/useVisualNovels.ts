import { VisualNovel, VisualNovelImage } from '@/database';
import {
    readonly,
    ref,
    shallowRef,
    toRef,
    watchEffect,
    type MaybeRefOrGetter
} from 'vue';
import { RendererProcessError } from '@/utils/error';
import { getVisualNovelById } from '@/utils/query';

function cache() {
    const novels = shallowRef<VisualNovel[]>([]);
    const isLoading = ref(false);

    async function reload() {
        try {
            isLoading.value = true;
            novels.value = await VisualNovel.findAll({
                include: VisualNovelImage
            });
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

export function useVisualNovel(id: MaybeRefOrGetter<Nullish<string>>) {
    const vndbid = toRef(id);
    const novel = shallowRef<VisualNovel | null>(null);
    const isLoading = ref(false);

    watchEffect(async () => {
        if (!vndbid.value) {
            novel.value = null;
            isLoading.value = false;
            return;
        }

        try {
            novel.value = null;
            isLoading.value = true;
            novel.value = await getVisualNovelById(vndbid.value);
        } catch (err) {
            novel.value = null;
            RendererProcessError.catch(err);
        } finally {
            isLoading.value = false;
        }
    });

    return {
        novel,
        isLoading: readonly(isLoading)
    };
}
