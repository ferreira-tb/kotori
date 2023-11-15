import { VNDB } from 'vndb-query';
import { DataTypes } from '@sequelize/core';
import { BaseImage, BaseModel } from '@/database/models/abstract';
import { Is, IsUrl } from '@sequelize/validator.js';
import type {
    BelongsToCreateAssociationMixin,
    BelongsToGetAssociationMixin,
    BelongsToSetAssociationMixin,
    CreationOptional,
    HasManyAddAssociationMixin,
    HasManyAddAssociationsMixin,
    HasManyCountAssociationsMixin,
    HasManyCreateAssociationMixin,
    HasManyGetAssociationsMixin,
    HasManyHasAssociationMixin,
    HasManyHasAssociationsMixin,
    HasManyRemoveAssociationMixin,
    HasManyRemoveAssociationsMixin,
    HasManySetAssociationsMixin,
    HasOneCreateAssociationMixin,
    HasOneGetAssociationMixin,
    HasOneSetAssociationMixin,
    NonAttribute
} from '@sequelize/core';
import {
    Attribute,
    Default,
    HasMany,
    HasOne,
    NotNull,
    PrimaryKey,
    Table,
    Unique
} from '@sequelize/core/decorators-legacy';

/**
 * @see https://tb.dev.br/vndb-query/api/types/ResponsePostVisualNovel.html
 */
@Table({ modelName: 'VisualNovel' })
export class VisualNovel extends BaseModel<VisualNovel> {
    @Attribute(DataTypes.TEXT)
    @Is(VNDB.regex.id.vn)
    @PrimaryKey
    public declare readonly id: string;

    @Attribute(DataTypes.JSON)
    @Default(() => [])
    @NotNull
    public declare readonly aliases: CreationOptional<string[]>;

    @Attribute(DataTypes.TEXT)
    public declare readonly alttitle: CreationOptional<string | null>;

    @Attribute(DataTypes.TEXT)
    public declare readonly description: CreationOptional<string | null>;

    @Attribute(DataTypes.INTEGER)
    @NotNull
    public declare readonly devstatus: 0 | 1 | 2;

    @HasOne(() => VisualNovelImage, {
        foreignKey: 'visualNovelId',
        inverse: {
            as: 'visualNovel'
        }
    })
    public declare readonly image?: NonAttribute<VisualNovelImage>;

    @Attribute(DataTypes.JSON)
    @Default(() => [])
    @NotNull
    public declare readonly languages: CreationOptional<string[]>;

    @Attribute(DataTypes.INTEGER)
    public declare readonly length: CreationOptional<1 | 2 | 3 | 4 | 5 | null>;

    @Attribute(DataTypes.INTEGER)
    public declare readonly lengthMinutes: CreationOptional<number | null>;

    @Attribute(DataTypes.INTEGER)
    @Default(0)
    @NotNull
    public declare readonly lengthVotes: CreationOptional<number>;

    @Attribute(DataTypes.TEXT)
    @NotNull
    public declare readonly olang: string;

    @Attribute(DataTypes.JSON)
    @Default(() => [])
    @NotNull
    public declare readonly platforms: CreationOptional<string[]>;

    @Attribute(DataTypes.FLOAT)
    public declare readonly rating: CreationOptional<number | null>;

    @Attribute(DataTypes.DATE)
    public declare readonly released: CreationOptional<Date | null>;

    @HasMany(() => VisualNovelScreenshot, {
        foreignKey: 'visualNovelId',
        inverse: {
            as: 'visualNovel'
        }
    })
    public declare readonly screenshots?: NonAttribute<VisualNovelScreenshot[]>;

    @Attribute(DataTypes.TEXT)
    @NotNull
    public declare readonly title: string;

    @Attribute(DataTypes.INTEGER)
    @Default(0)
    @NotNull
    public declare readonly votecount: CreationOptional<number>;

    public declare readonly createVisualNovelImage: HasOneCreateAssociationMixin<
        VisualNovelImage,
        'visualNovelId'
    >;
    public declare readonly getVisualNovelImage: HasOneGetAssociationMixin<VisualNovelImage>;
    public declare readonly setVisualNovelImage: HasOneSetAssociationMixin<
        VisualNovelImage,
        VisualNovelImage['id']
    >;

    public declare readonly addVisualNovelScreenshot: HasManyAddAssociationMixin<
        VisualNovelScreenshot,
        VisualNovelScreenshot['id']
    >;
    public declare readonly addVisualNovelScreenshots: HasManyAddAssociationsMixin<
        VisualNovelScreenshot,
        VisualNovelScreenshot['id']
    >;
    public declare readonly countVisualNovelScreenshots: HasManyCountAssociationsMixin<VisualNovelScreenshot>;
    public declare readonly createVisualNovelScreenshot: HasManyCreateAssociationMixin<
        VisualNovelScreenshot,
        'visualNovelId'
    >;
    public declare readonly getVisualNovelScreenshots: HasManyGetAssociationsMixin<VisualNovelScreenshot>;
    public declare readonly hasVisualNovelScreenshot: HasManyHasAssociationMixin<
        VisualNovelScreenshot,
        VisualNovelScreenshot['id']
    >;
    public declare readonly hasVisualNovelScreenshots: HasManyHasAssociationsMixin<
        VisualNovelScreenshot,
        VisualNovelScreenshot['id']
    >;
    public declare readonly removeVisualNovelScreenshot: HasManyRemoveAssociationMixin<
        VisualNovelScreenshot,
        VisualNovelScreenshot['id']
    >;
    public declare readonly removeVisualNovelScreenshots: HasManyRemoveAssociationsMixin<
        VisualNovelScreenshot,
        VisualNovelScreenshot['id']
    >;
    public declare readonly setVisualNovelScreenshots: HasManySetAssociationsMixin<
        VisualNovelScreenshot,
        VisualNovelScreenshot['id']
    >;
}

/**
 * @see https://tb.dev.br/vndb-query/api/types/ResponsePostVisualNovelImage.html
 */
@Table({ modelName: 'VisualNovelImage' })
export class VisualNovelImage extends BaseImage<VisualNovelImage> {
    @Attribute(DataTypes.TEXT)
    @Is(VNDB.regex.id.image.vn)
    @PrimaryKey
    public declare readonly id: string;

    @Attribute(DataTypes.TEXT)
    @Is(VNDB.regex.id.vn)
    @NotNull
    @Unique
    public declare readonly visualNovelId: string;

    /** Defined by {@link VisualNovel.image} */
    public declare readonly visualNovel?: NonAttribute<VisualNovel>;

    public declare readonly createVisualNovel: BelongsToCreateAssociationMixin<VisualNovel>;
    public declare readonly getVisualNovel: BelongsToGetAssociationMixin<VisualNovel>;
    public declare readonly setVisualNovel: BelongsToSetAssociationMixin<
        VisualNovel,
        VisualNovelImage['visualNovelId']
    >;
}

/**
 * @see https://tb.dev.br/vndb-query/api/types/ResponsePostVisualNovelScreenshot.html
 */
@Table({ modelName: 'VisualNovelScreenshot' })
export class VisualNovelScreenshot extends BaseImage<VisualNovelScreenshot> {
    @Attribute(DataTypes.TEXT)
    @Is(VNDB.regex.id.image.screenshot)
    @PrimaryKey
    public declare readonly id: string;

    @Attribute(DataTypes.TEXT)
    @Is(VNDB.regex.id.vn)
    @NotNull
    public declare readonly visualNovelId: string;

    @Attribute(DataTypes.TEXT)
    @IsUrl
    @NotNull
    public declare readonly thumbnail: string;

    @Attribute(DataTypes.JSON)
    @NotNull
    public declare readonly thumbnailDims: [number, number];

    /** Defined by {@link VisualNovel.screenshots} */
    public declare readonly visualNovel?: NonAttribute<VisualNovel>;

    public declare readonly createVisualNovel: BelongsToCreateAssociationMixin<VisualNovel>;
    public declare readonly getVisualNovel: BelongsToGetAssociationMixin<VisualNovel>;
    public declare readonly setVisualNovel: BelongsToSetAssociationMixin<
        VisualNovel,
        VisualNovelScreenshot['visualNovelId']
    >;
}
