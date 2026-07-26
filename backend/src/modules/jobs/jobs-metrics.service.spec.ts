/**
 * Issue #1314 — JobsMetricsService: failed-job metrics.
 */
import { Test, TestingModule } from '@nestjs/testing';
import { getDataSourceToken } from '@nestjs/typeorm';
import { HttpMetricsService } from '../metrics/http-metrics.service';
import { JobsMetricsService } from './jobs-metrics.service';

const mockDataSource = {
  driver: { master: { totalCount: 3, idleCount: 2, waitingCount: 0 } },
  options: { poolSize: 5 },
  query: jest.fn(),
};

describe('JobsMetricsService', () => {
  let jobsMetrics: JobsMetricsService;
  let httpMetrics: HttpMetricsService;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        JobsMetricsService,
        HttpMetricsService,
        { provide: getDataSourceToken(), useValue: mockDataSource },
      ],
    }).compile();

    jobsMetrics = module.get(JobsMetricsService);
    httpMetrics = module.get(HttpMetricsService);
  });

  it('starts at zero for an unseen queue/job combination', async () => {
    expect(await jobsMetrics.getFailedCount('background-jobs', 'sample-echo')).toBe(0);
  });

  it('increments the failed counter per queue/job label pair', async () => {
    jobsMetrics.recordFailure('background-jobs', 'sample-echo');
    jobsMetrics.recordFailure('background-jobs', 'sample-echo');
    jobsMetrics.recordFailure('background-jobs', 'other-job');

    expect(await jobsMetrics.getFailedCount('background-jobs', 'sample-echo')).toBe(2);
    expect(await jobsMetrics.getFailedCount('background-jobs', 'other-job')).toBe(1);
  });

  it('exposes tycoon_jobs_failed_total on the shared metrics registry', async () => {
    jobsMetrics.recordFailure('background-jobs', 'sample-echo');
    const text = await httpMetrics.getMetricsText();
    expect(text).toContain('tycoon_jobs_failed_total');
  });

  it('falls back to "unknown" when no job name is available', async () => {
    jobsMetrics.recordFailure('background-jobs', '');
    expect(await jobsMetrics.getFailedCount('background-jobs', 'unknown')).toBe(1);
  });
});
