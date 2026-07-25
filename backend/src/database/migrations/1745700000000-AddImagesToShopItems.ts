import { MigrationInterface, QueryRunner } from 'typeorm';

export class AddImagesToShopItems1745700000000 implements MigrationInterface {
  public async up(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`
      ALTER TABLE "shop_items"
      ADD COLUMN "images" json DEFAULT '[]'
    `);
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    await queryRunner.query(`
      ALTER TABLE "shop_items"
      DROP COLUMN "images"
    `);
  }
}
