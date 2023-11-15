import { ipcSend } from '@/utils/ipc';

export class RendererProcessError extends Error {
    public override readonly name = 'RendererProcessError';

    public static catch(err: unknown): void {
        if (err instanceof Error) {
            ipcSend('error:catch', err.name, err.message, err.stack);
        }
    }
}
