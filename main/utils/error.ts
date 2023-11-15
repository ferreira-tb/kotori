import * as fs from 'node:fs/promises';
import * as path from 'node:path';
import { app, dialog } from 'electron';

export class KotoriError extends Error {
    public override name = 'KotoriError';

    public static async catch(err: unknown) {
        if (!(err instanceof Error)) return;

        const message = err.stack ?? err.message;
        dialog.showErrorBox(err.name, message);

        const date = new Date().toLocaleString('en-US');
        const kotori = app.getVersion();
        const electron = process.versions.electron;
        const chrome = process.versions.chrome;

        const log = path.join(app.getPath('userData'), 'error.log');
        const content = `${date}\nKotori: ${kotori} Electron: ${electron} Chrome: ${chrome}\n${message}\n\n`;
        await fs.appendFile(log, content, { encoding: 'utf-8' });
    }
}
