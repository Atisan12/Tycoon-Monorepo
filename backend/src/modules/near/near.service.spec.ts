import { Test, TestingModule } from '@nestjs/testing';
import { ConfigService } from '@nestjs/config';
import { NearService } from './near.service';

describe('NearService', () => {
  let service: NearService;

  const mockConfigService = {
    get: jest.fn((key: string) => {
      if (key === 'near.rpcEndpoints')
        return ['http://rpc1.test', 'http://rpc2.test'];
      if (key === 'near.timeoutMs') return 1000;
      return null;
    }),
  };

  beforeEach(async () => {
    global.fetch = jest.fn();
    (global as any).AbortController = jest.fn(() => ({
      abort: jest.fn(),
      signal: 'mock-signal',
    }));

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        NearService,
        { provide: ConfigService, useValue: mockConfigService },
      ],
    }).compile();

    service = module.get<NearService>(NearService);
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  it('should be defined', () => {
    expect(service).toBeDefined();
  });

  it('should successfully make a view call on the first endpoint', async () => {
    const mockRpcResponse = {
      json: jest.fn().mockResolvedValue({
        result: {
          result: Array.from(Buffer.from(JSON.stringify({ success: true }))),
        },
      }),
      ok: true,
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(mockRpcResponse);

    const result = await service.view('contract.testnet', 'get_status', {});

    expect(global.fetch).toHaveBeenCalledTimes(1);
    expect(global.fetch).toHaveBeenCalledWith(
      'http://rpc1.test',
      expect.objectContaining({ method: 'POST' }),
    );
    expect(result).toEqual({ success: true });
  });

  it('should rotate endpoint on network failure and succeed on second', async () => {
    // First call fails (e.g. ECONNREFUSED)
    (global.fetch as jest.Mock).mockRejectedValueOnce(
      new Error('Network Error'),
    );

    // Second call succeeds
    const mockRpcResponse = {
      json: jest.fn().mockResolvedValue({
        result: {
          result: Array.from(Buffer.from(JSON.stringify({ amount: 100 }))),
        },
      }),
      ok: true,
    };
    (global.fetch as jest.Mock).mockResolvedValueOnce(mockRpcResponse);

    const result = await service.view('contract.testnet', 'get_balance', {});

    expect(global.fetch).toHaveBeenCalledTimes(2);
    expect(global.fetch).toHaveBeenNthCalledWith(
      1,
      'http://rpc1.test',
      expect.any(Object),
    );
    expect(global.fetch).toHaveBeenNthCalledWith(
      2,
      'http://rpc2.test',
      expect.any(Object),
    );
    expect(result).toEqual({ amount: 100 });
  });

  it('should throw an error if all endpoints fail', async () => {
    (global.fetch as jest.Mock).mockRejectedValue(new Error('Timeout Error'));

    await expect(
      service.view('contract.testnet', 'get_status', {}),
    ).rejects.toThrow(/All NEAR RPC endpoints failed/);

    expect(global.fetch).toHaveBeenCalledTimes(2); // Since there are 2 endpoints configured
  });

  it('should not rotate if it is a contract error (FunctionCallError)', async () => {
    (global.fetch as jest.Mock).mockResolvedValueOnce({
      ok: true,
      json: jest.fn().mockResolvedValue({
        error: { message: 'FunctionCallError: method not found' },
      }),
    });

    await expect(
      service.view('contract.testnet', 'bad_method', {}),
    ).rejects.toThrow(/FunctionCallError: method not found/);

    expect(global.fetch).toHaveBeenCalledTimes(1); // No rotation for contract error!
  });

  describe('circuit breaker', () => {
    it('opens the circuit after 3 consecutive failures', async () => {
      (global.fetch as jest.Mock).mockRejectedValue(new Error('ECONNREFUSED'));

      // 3 failures across 2 endpoints = circuit opens mid-way
      await expect(service.rpcCall('status', {})).rejects.toThrow();

      expect((service as any).circuitState).toBe('OPEN');
    });

    it('throws ServiceUnavailableException with NEAR_SERVICE_UNAVAILABLE when circuit is open', async () => {
      // Force circuit open
      (service as any).circuitState = 'OPEN';
      (service as any).openedAt = Date.now();

      await expect(service.rpcCall('status', {})).rejects.toMatchObject({
        response: expect.objectContaining({ code: 'NEAR_SERVICE_UNAVAILABLE' }),
      });

      // fetch should NOT have been called (fail-fast)
      expect(global.fetch).not.toHaveBeenCalled();
    });

    it('resets the circuit to CLOSED after a successful probe in HALF_OPEN state', async () => {
      (service as any).circuitState = 'HALF_OPEN';
      (service as any).consecutiveFailures = 3;

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: jest.fn().mockResolvedValue({ result: null }),
      });

      await service.rpcCall('status', {});

      expect((service as any).circuitState).toBe('CLOSED');
      expect((service as any).consecutiveFailures).toBe(0);
    });

    it('transitions from OPEN to HALF_OPEN after recovery timeout', async () => {
      (service as any).circuitState = 'OPEN';
      // Simulate openedAt 31 seconds ago
      (service as any).openedAt = Date.now() - 31_000;

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: jest.fn().mockResolvedValue({ result: null }),
      });

      // Should allow the call through (HALF_OPEN probe)
      await service.rpcCall('status', {});
      expect(global.fetch).toHaveBeenCalledTimes(1);
    });
  });
});
