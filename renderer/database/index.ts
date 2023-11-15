import { Sequelize } from '@sequelize/core';
import { ipcSendSync } from '@/utils/ipc';
import { join } from 'node:path';
import {
    VisualNovel,
    VisualNovelImage,
    VisualNovelScreenshot
} from '@/database/models';

export const sequelize = new Sequelize({
    dialect: 'sqlite',
    storage: join(ipcSendSync('app(sync):user-data'), 'kotori.db'),
    logging: false,
    models: [VisualNovel, VisualNovelImage, VisualNovelScreenshot]
});

export * from '@/database/models';
