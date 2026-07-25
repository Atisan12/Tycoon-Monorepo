import { Test, TestingModule } from '@nestjs/testing';
import { JwtService } from '@nestjs/jwt';
import { PerkBoostGateway } from './perk-boost.gateway';
import { PerksBoostsEvents } from '../services/perks-boosts-events.service';

describe('PerkBoostGateway - auth (#1296)', () => {
  let gateway: PerkBoostGateway;
  let jwtService: { verifyAsync: jest.Mock };

  function makeSocket(overrides: Record<string, any> = {}) {
    return {
      id: 'socket-1',
      handshake: { auth: {}, headers: {}, query: {} },
      data: {},
      join: jest.fn(),
      disconnect: jest.fn(),
      ...overrides,
    };
  }

  beforeEach(async () => {
    jwtService = { verifyAsync: jest.fn() };

    const module: TestingModule = await Test.createTestingModule({
      providers: [
        PerkBoostGateway,
        {
          provide: PerksBoostsEvents,
          useValue: { events$: { subscribe: jest.fn() } },
        },
        { provide: JwtService, useValue: jwtService },
      ],
    }).compile();

    gateway = module.get<PerkBoostGateway>(PerkBoostGateway);
  });

  it('disconnects sockets with no token', async () => {
    const socket = makeSocket();
    await gateway.handleConnection(socket as any);
    expect(socket.disconnect).toHaveBeenCalledWith(true);
    expect(socket.join).not.toHaveBeenCalled();
  });

  it('disconnects sockets with an invalid/expired token', async () => {
    jwtService.verifyAsync.mockRejectedValue(new Error('invalid token'));
    const socket = makeSocket({
      handshake: { auth: { token: 'bad-token' }, headers: {}, query: {} },
    });

    await gateway.handleConnection(socket as any);
    expect(socket.disconnect).toHaveBeenCalledWith(true);
    expect(socket.join).not.toHaveBeenCalled();
  });

  it('joins the room derived from the verified token, not a client-supplied userId', async () => {
    jwtService.verifyAsync.mockResolvedValue({ sub: 42 });
    const socket = makeSocket({
      handshake: {
        auth: { token: 'good-token' },
        headers: {},
        query: { userId: '999' },
      },
    });

    await gateway.handleConnection(socket as any);
    expect(socket.join).toHaveBeenCalledWith('user_42');
    expect(socket.disconnect).not.toHaveBeenCalled();
  });

  it('accepts a bearer token from the Authorization header as a fallback', async () => {
    jwtService.verifyAsync.mockResolvedValue({ sub: 7 });
    const socket = makeSocket({
      handshake: {
        auth: {},
        headers: { authorization: 'Bearer good-token' },
        query: {},
      },
    });

    await gateway.handleConnection(socket as any);
    expect(socket.join).toHaveBeenCalledWith('user_7');
  });
});
