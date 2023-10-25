import { VNDB } from 'vndb-query';
import { DataTypes } from '@sequelize/core';
import { BaseModel } from '@/database/models/abstract';
import { Is } from '@sequelize/validator.js';
import type { CreationOptional } from '@sequelize/core';
import {
	Attribute,
	Default,
	NotNull,
	PrimaryKey,
	Table
} from '@sequelize/core/decorators-legacy';

/**
 * @see https://tb.dev.br/vndb-query/api/types/ResponsePostVisualNovel.html
 */
@Table({ modelName: 'VisualNovel' })
export class VisualNovel extends BaseModel<VisualNovel> {
	@Attribute(DataTypes.TEXT)
	@Is(VNDB.regex.vn)
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

	@Attribute(DataTypes.JSON)
	@Default(() => [])
	@NotNull
	public declare readonly languages: CreationOptional<string[]>;

	@Attribute(DataTypes.INTEGER)
	public declare readonly length: CreationOptional<1 | 2 | 3 | 4 | 5 | null>;

	@Attribute(DataTypes.INTEGER)
	public declare readonly length_minutes: CreationOptional<number | null>;

	@Attribute(DataTypes.INTEGER)
	@Default(0)
	@NotNull
	public declare readonly length_votes: CreationOptional<number>;

	@Attribute(DataTypes.TEXT)
	@NotNull
	public declare readonly olang: string;

	@Attribute(DataTypes.JSON)
	@Default(() => [])
	@NotNull
	public declare readonly platforms: CreationOptional<string[]>;

	@Attribute(DataTypes.REAL)
	public declare readonly rating: CreationOptional<number | null>;

	@Attribute(DataTypes.DATE)
	public declare readonly released: CreationOptional<Date | null>;

	@Attribute(DataTypes.TEXT)
	@NotNull
	public declare readonly title: string;

	@Attribute(DataTypes.INTEGER)
	@Default(0)
	@NotNull
	public declare readonly votecount: CreationOptional<number>;
}
