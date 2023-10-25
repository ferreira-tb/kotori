import { Sequelize } from '@sequelize/core';
import { ipcRenderer } from 'electron';
import { join } from 'node:path';
import { VisualNovel } from '@/database/models';

export const sequelize = new Sequelize({
	dialect: 'sqlite',
	storage: join(ipcRenderer.sendSync('app(sync):user-data'), 'kotori.db'),
	logging: false,
	models: [VisualNovel]
});

export * from '@/database/models';
