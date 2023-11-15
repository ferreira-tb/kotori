import { z } from 'zod';
import { VNDB } from 'vndb-query';

export const parser = {
    vn: () => {
        return z.object({
            id: z.string().regex(VNDB.regex.id.vn),
            title: z.string(),
            alttitle: z.string(),
            olang: z.string(),
            devstatus: z.union([z.literal(0), z.literal(1), z.literal(2)]),
            image: z.object({
                id: z.string(),
                url: z.string().url(),
                dims: z.tuple([z.number().int(), z.number().int()]),
                sexual: z.number().min(0).max(2),
                violence: z.number().min(0).max(2),
                votecount: z.number().int()
            })
        });
    }
};
