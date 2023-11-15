import { useElementSize as original } from '@vueuse/core';
import type { ElementSize, UseResizeObserverOptions } from '@vueuse/core';

/** Exactly like the `useElementSize` from vueuse, but already with { box: 'border-box' }.. */
export function useElementSize(
    element: Parameters<typeof original>[0],
    initialSize: Partial<ElementSize> = {},
    options: UseResizeObserverOptions = {}
) {
    return original(
        element,
        { width: 0, height: 0, ...initialSize },
        { box: 'border-box', ...options }
    );
}
