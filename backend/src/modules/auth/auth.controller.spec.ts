import { Test, TestingModule } from '@nestjs/testing';
import { Reflector } from '@nestjs/core';
import { THROTTLER_LIMIT, THROTTLER_TTL } from '@nestjs/throttler';
import { AuthController } from './auth.controller';
import { AuthService } from './auth.service';
import { UsersService } from '../users/users.service';

const mockAuthService = {
  login: jest.fn(),
  refreshTokens: jest.fn(),
  walletLogin: jest.fn(),
  logout: jest.fn(),
};

const mockUsersService = {
  create: jest.fn(),
};

describe('AuthController – throttler configuration', () => {
  let controller: AuthController;
  let reflector: Reflector;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      controllers: [AuthController],
      providers: [
        { provide: AuthService, useValue: mockAuthService },
        { provide: UsersService, useValue: mockUsersService },
        Reflector,
      ],
    }).compile();

    controller = module.get<AuthController>(AuthController);
    reflector = module.get<Reflector>(Reflector);
  });

  const getThrottle = (handler: (...args: any[]) => any) => {
    // @Throttle stores metadata under the throttler keys
    const limit = Reflect.getMetadata(THROTTLER_LIMIT, handler);
    const ttl = Reflect.getMetadata(THROTTLER_TTL, handler);
    return { limit, ttl };
  };

  it('login is rate-limited to 5 requests per 60 000 ms', () => {
    const meta = Reflect.getMetadata('throttler', controller.login);
    expect(meta).toBeDefined();
    expect(meta[0]).toMatchObject({ limit: 5, ttl: 60000 });
  });

  it('register is rate-limited to 10 requests per 60 000 ms', () => {
    const meta = Reflect.getMetadata('throttler', controller.register);
    expect(meta).toBeDefined();
    expect(meta[0]).toMatchObject({ limit: 10, ttl: 60000 });
  });

  it('wallet-login is rate-limited to 20 requests per 60 000 ms', () => {
    const meta = Reflect.getMetadata('throttler', controller.walletLogin);
    expect(meta).toBeDefined();
    expect(meta[0]).toMatchObject({ limit: 20, ttl: 60000 });
  });

  it('refresh endpoint has no throttle override (falls back to global 100/min)', () => {
    const meta = Reflect.getMetadata('throttler', controller.refresh);
    expect(meta).toBeUndefined();
  });

  it('logout endpoint has no throttle override (protected by JwtAuthGuard)', () => {
    const meta = Reflect.getMetadata('throttler', controller.logout);
    expect(meta).toBeUndefined();
  });
});
