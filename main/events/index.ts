import { app, ipcMain } from 'electron';
import { KotoriError } from '../utils/error';

export function setAppEvents() {
	ipcMain.on('app(sync):user-data', (e) => {
		e.returnValue = app.getPath('userData');
	});

	ipcMain.handle('app:version', () => app.getVersion());

	ipcMain.on(
		'error:catch',
		(_e, name: string, message: string, stack?: string) => {
			const err = new KotoriError(message);
			err.name = name;
			err.message = message;

			delete err.stack;
			if (stack) err.stack = stack;

			KotoriError.catch(err);
		}
	);
}
