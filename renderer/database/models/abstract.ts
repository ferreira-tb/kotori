import { DataTypes, Model } from '@sequelize/core';
import { IsUrl } from '@sequelize/validator.js';
import {
	Attribute,
	Default,
	DeletedAt,
	NotNull,
	PrimaryKey,
	Table
} from '@sequelize/core/decorators-legacy';
import type {
	CreationOptional,
	InferAttributes,
	InferCreationAttributes
} from '@sequelize/core';

@Table.Abstract({ timestamps: true })
export abstract class BaseModel<T extends Model = Model> extends Model<
	InferAttributes<T>,
	InferCreationAttributes<T>
> {
	@DeletedAt
	public declare readonly deletedAt: Date | null;

	public declare readonly createdAt: CreationOptional<Date>;
	public declare readonly updatedAt: CreationOptional<Date>;
}

@Table.Abstract({ timestamps: true })
export abstract class BaseImage<T extends Model = Model> extends BaseModel<T> {
	@Attribute(DataTypes.TEXT)
	@PrimaryKey
	public declare readonly id: string;

	@Attribute(DataTypes.BLOB)
	public declare readonly blob: CreationOptional<Blob | null>;

	@Attribute(DataTypes.JSON)
	@NotNull
	public declare readonly dims: [number, number];

	@Attribute(DataTypes.INTEGER)
	@NotNull
	public declare readonly sexual: 0 | 1 | 2;

	@Attribute(DataTypes.TEXT)
	@IsUrl
	@NotNull
	public declare readonly url: string;

	@Attribute(DataTypes.INTEGER)
	@NotNull
	public declare readonly violence: 0 | 1 | 2;

	@Attribute(DataTypes.INTEGER)
	@Default(0)
	@NotNull
	public declare readonly votecount: number;
}
