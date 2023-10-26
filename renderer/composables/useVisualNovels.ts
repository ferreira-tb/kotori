import { VisualNovel, VisualNovelImage } from '@/database';
import { useAsyncState } from './useAsyncState';

export function useVisualNovels() {
	const { state: novels } = useAsyncState(
		() => VisualNovel.findAll({ include: VisualNovelImage }),
		[]
	);

	return {
		novels
	};
}
