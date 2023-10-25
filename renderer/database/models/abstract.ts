import { Model } from '@sequelize/core';
import { DeletedAt, Table } from '@sequelize/core/decorators-legacy';
import type {
	CreationOptional,
	InferAttributes,
	InferCreationAttributes
} from '@sequelize/core';

@Table.Abstract({
	timestamps: true
})
export abstract class BaseModel<T extends Model = Model> extends Model<
	InferAttributes<T>,
	InferCreationAttributes<T>
> {
	@DeletedAt
	public declare readonly deletedAt: Date | null;

	public declare readonly createdAt: CreationOptional<Date>;
	public declare readonly updatedAt: CreationOptional<Date>;
}
