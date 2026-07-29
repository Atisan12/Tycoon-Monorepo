import { Module } from '@nestjs/common';
import { TypeOrmModule } from '@nestjs/typeorm';
import { Chance } from './entities/chance.entity';
import { ChanceService } from './chance.service';
import { ChanceController } from './chance.controller';
import { ChanceValidationFilter, ChanceExceptionFilter } from './filters/chance-validation.filter';
import { ChanceObservabilityService } from './chance-observability.service';
import { LoggerModule } from '../../common/logger/logger.module';
import { RANDOM_PROVIDER, SecureRandomProvider } from '../../common/random-provider';

@Module({
  imports: [TypeOrmModule.forFeature([Chance], LoggerModule)],
  providers: [
    ChanceService,
    ChanceValidationFilter,
    ChanceExceptionFilter,
    ChanceObservabilityService,
    { provide: RANDOM_PROVIDER, useClass: SecureRandomProvider },
  ],
  controllers: [ChanceController],
  exports: [TypeOrmModule, ChanceObservabilityService],
})
export class ChanceModule {}
