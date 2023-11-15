import { ipcRenderer } from 'electron';

export async function ipcInvoke(channel: 'app:version'): Promise<string>;
export async function ipcInvoke(
    channel: string,
    ...args: any[]
): Promise<unknown> {
    const response: unknown = await ipcRenderer.invoke(channel, ...args);
    return response;
}

export function ipcSend(
    channel: 'error:catch',
    name: string,
    message: string,
    stack?: string
): void;
export function ipcSend(channel: string, ...args: any[]): void {
    ipcRenderer.send(channel, ...args);
}

export function ipcSendSync(channel: 'app(sync):user-data'): string;
export function ipcSendSync(channel: string, ...args: any[]): any {
    return ipcRenderer.sendSync(channel, ...args);
}
