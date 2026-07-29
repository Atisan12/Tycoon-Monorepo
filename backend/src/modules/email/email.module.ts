import { Module } from '@nestjs/common';
import { EmailService } from './email.service';
import { EmailProcessor } from './email.processor';
import { JobsModule } from '../jobs/jobs.module';
import { BullModule } from '@nestjs/bullmq';

/**
 * EmailModule
 *
 * Queue: `email-queue`
 *   - Max attempts : 3  (exponential backoff, base delay 5 000 ms)
 *   - On exhaustion: job is kept in the failed set (`removeOnFail: false`) and
 *     the processor emits a structured error log referencing the recipient and
 *     template.  Use the BullMQ dashboard (Bull Board) or the Redis CLI to
 *     inspect / replay DLQ entries:
 *       LRANGE bull:email-queue:failed 0 -1
 */
@Module({
  imports: [
    JobsModule,
    BullModule.registerQueue({
      name: 'email-queue',
    }),
  ],
  providers: [EmailService, EmailProcessor],
  exports: [EmailService],
})
export class EmailModule {}
