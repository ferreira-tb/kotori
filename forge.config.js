const path = require('node:path');
const asar = require('@electron/asar');
const { existsSync: exists } = require('node:fs');

/** @type {import('@electron-forge/shared-types').ForgeConfig} */
const config = {
    packagerConfig: {
        asar: true,
        name: 'kotori',
        executableName: 'kotori',
        overwrite: true,
        icon: path.join(__dirname, 'public/favicon'),
        prune: true,
        ignore: /^\/(?!\.vite|node_modules|package\.json)/
    },
    rebuildConfig: {},
    makers: [
        {
            name: '@electron-forge/maker-zip'
        }
    ],
    plugins: [
        {
            name: '@electron-forge/plugin-vite',
            config: {
                build: [
                    {
                        entry: 'main/index.ts',
                        config: 'vite.main.config.ts'
                    }
                ],
                renderer: [
                    {
                        name: 'main_window',
                        config: 'vite.renderer.config.ts'
                    }
                ]
            }
        }
    ],
    hooks: {
        generateAssets: async () => {
            // https://github.com/ferreira-tb/yukari/issues/90
            const electronAsar = path.join(
                __dirname,
                'node_modules/electron/dist/resources/electron.asar'
            );
            if (!exists(electronAsar)) {
                const src = path.join(
                    __dirname,
                    'node_modules/@sequelize/core/node_modules/bnf-parser'
                );
                await asar.createPackage(src, electronAsar);
            }
        }
    }
};

module.exports = config;
