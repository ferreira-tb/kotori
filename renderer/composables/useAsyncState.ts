import { RendererProcessError } from '@/utils/error';
import {
	useAsyncState as original,
	type UseAsyncStateOptions
} from '@vueuse/core';

export function useAsyncState<
	Data,
	Params extends any[] = [],
	Shallow extends boolean = true
>(
	promise: Promise<Data> | ((...args: Params) => Promise<Data>),
	initialState: Data,
	options?: UseAsyncStateOptions<Shallow, Data>
) {
	const defaultOptions: UseAsyncStateOptions<Shallow, Data> = {
		immediate: true,
		resetOnExecute: true,
		shallow: true as Shallow,
		throwError: true,
		onError: (err) => RendererProcessError.catch(err)
	};

	return original(promise, initialState, { ...defaultOptions, ...options });
}
