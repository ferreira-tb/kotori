import { useVisualNovels } from '@/composables';
import { VisualNovel, VisualNovelImage } from '@/database';
import { VNDB } from 'vndb-query';
import { triggerRef } from 'vue';
import { fields } from '@/utils/fields';
import { parser } from '@/utils/parser';
import { RendererProcessError } from '@/utils/error';

export async function getVisualNovelsByTitle(title: string) {
    try {
        const vndb = new VNDB();
        const { novels } = useVisualNovels();

        let { results } = await vndb.search('vn', title, {
            results: 50,
            fields: fields.vn()
        });

        results = results.filter((value) => {
            return novels.value.every(({ id }) => id !== value.id.trim());
        });

        const schema = parser.vn();
        const models = await VisualNovel.sequelize.transaction(async () => {
            return await Promise.all(
                results.map((value) => {
                    return VisualNovel.sequelize.transaction(async () => {
                        const vn = schema.parse(value);
                        return await VisualNovel.create(vn, {
                            include: VisualNovelImage
                        });
                    });
                })
            );
        });

        novels.value.push(...models);
        triggerRef(novels);
    } catch (err) {
        RendererProcessError.catch(err);
    }
}

export async function getVisualNovelById(
    id: string
): Promise<VisualNovel | null> {
    try {
        const vndb = new VNDB();
        const { novels } = useVisualNovels();

        let novel: Nullish<VisualNovel> = novels.value.find(
            (novel) => novel.id === id
        );
        if (novel) {
            return novel;
        } else {
            const model = await VisualNovel.findByPk(id, {
                include: VisualNovelImage
            });
            if (model) {
                novels.value.push(model);
                triggerRef(novels);
                return model;
            }
        }

        console.log(id);
        const { results } = await vndb.search('vn', id, {
            results: 1,
            fields: fields.vn()
        });

        if (results.length === 0) return null;

        const schema = parser.vn();
        const model = await VisualNovel.sequelize.transaction(async () => {
            return await VisualNovel.create(schema.parse(results[0]), {
                include: VisualNovelImage
            });
        });

        novels.value.push(model);
        triggerRef(novels);

        return model;
    } catch (err) {
        RendererProcessError.catch(err);
        return null;
    }
}
