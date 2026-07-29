import { describe, it, expect } from 'vitest';
import {
  loginSchema,
  adminLoginSchema,
  walletLoginSchema,
  joinRoomSchema,
  inviteTokenSchema,
  displayNameSchema,
  gameSettingsSchema,
  type LoginFormValues,
  type AdminLoginFormValues,
} from './schemas';

describe('Validation Schemas', () => {
  describe('loginSchema', () => {
    it('validates correct login credentials', () => {
      const result = loginSchema.safeParse({
        email: 'user@example.com',
        password: 'mypassword123',
      });

      expect(result.success).toBe(true);
      if (result.success) {
        const data: LoginFormValues = result.data;
        expect(data.email).toBe('user@example.com');
        expect(data.password).toBe('mypassword123');
      }
    });

    it('requires email field (IsNotEmpty parity)', () => {
      const result = loginSchema.safeParse({
        email: '',
        password: 'password123',
      });

      expect(result.success).toBe(false);
      if (!result.success) {
        const emailError = result.error.issues.find((i) => i.path[0] === 'email');
        expect(emailError).toBeDefined();
      }
    });

    it('requires valid email format (IsEmail parity)', () => {
      const result = loginSchema.safeParse({
        email: 'not-an-email',
        password: 'password123',
      });

      expect(result.success).toBe(false);
      if (!result.success) {
        const emailError = result.error.issues.find((i) => i.path[0] === 'email');
        expect(emailError).toBeDefined();
        expect(emailError?.message).toContain('valid email');
      }
    });

    it('requires password field (IsNotEmpty parity)', () => {
      const result = loginSchema.safeParse({
        email: 'user@example.com',
        password: '',
      });

      expect(result.success).toBe(false);
      if (!result.success) {
        const passwordError = result.error.issues.find((i) => i.path[0] === 'password');
        expect(passwordError).toBeDefined();
      }
    });

    it('requires both fields', () => {
      const result = loginSchema.safeParse({
        email: '',
        password: '',
      });

      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.issues.length).toBeGreaterThanOrEqual(2);
      }
    });

    it('rejects missing email field', () => {
      const result = loginSchema.safeParse({
        password: 'password123',
      });

      expect(result.success).toBe(false);
    });

    it('rejects missing password field', () => {
      const result = loginSchema.safeParse({
        email: 'user@example.com',
      });

      expect(result.success).toBe(false);
    });

    it('handles whitespace-only email', () => {
      const result = loginSchema.safeParse({
        email: '   ',
        password: 'password123',
      });

      expect(result.success).toBe(false);
    });

    it('handles whitespace-only password', () => {
      const result = loginSchema.safeParse({
        email: 'user@example.com',
        password: '   ',
      });

      expect(result.success).toBe(false);
    });

    it('accepts various valid email formats', () => {
      const emails = [
        'user@example.com',
        'test.user@example.co.uk',
        'user+tag@example.com',
        'user@subdomain.example.com',
      ];

      emails.forEach((email) => {
        const result = loginSchema.safeParse({
          email,
          password: 'password123',
        });

        expect(result.success).toBe(true, `Email "${email}" should be valid`);
      });
    });
  });

  describe('adminLoginSchema', () => {
    it('validates correct admin login credentials', () => {
      const result = adminLoginSchema.safeParse({
        email: 'admin@example.com',
        password: 'adminpass123',
      });

      expect(result.success).toBe(true);
      if (result.success) {
        const data: AdminLoginFormValues = result.data;
        expect(data.email).toBe('admin@example.com');
        expect(data.password).toBe('adminpass123');
      }
    });

    it('enforces MinLength(6) password requirement parity', () => {
      const result = adminLoginSchema.safeParse({
        email: 'admin@example.com',
        password: '12345',
      });

      expect(result.success).toBe(false);
      if (!result.success) {
        const passwordError = result.error.issues.find((i) => i.path[0] === 'password');
        expect(passwordError).toBeDefined();
        expect(passwordError?.message).toContain('6 characters');
      }
    });

    it('accepts exactly 6 character password (minimum)', () => {
      const result = adminLoginSchema.safeParse({
        email: 'admin@example.com',
        password: '123456',
      });

      expect(result.success).toBe(true);
    });

    it('accepts passwords longer than 6 characters', () => {
      const result = adminLoginSchema.safeParse({
        email: 'admin@example.com',
        password: 'longsecurepassword',
      });

      expect(result.success).toBe(true);
    });

    it('requires email field', () => {
      const result = adminLoginSchema.safeParse({
        email: '',
        password: 'password123',
      });

      expect(result.success).toBe(false);
      if (!result.success) {
        const emailError = result.error.issues.find((i) => i.path[0] === 'email');
        expect(emailError).toBeDefined();
      }
    });

    it('requires valid email format', () => {
      const result = adminLoginSchema.safeParse({
        email: 'invalid-email',
        password: 'password123',
      });

      expect(result.success).toBe(false);
      if (!result.success) {
        const emailError = result.error.issues.find((i) => i.path[0] === 'email');
        expect(emailError).toBeDefined();
      }
    });

    it('rejects empty password', () => {
      const result = adminLoginSchema.safeParse({
        email: 'admin@example.com',
        password: '',
      });

      expect(result.success).toBe(false);
    });

    it('differentiates from loginSchema by enforcing password MinLength(6)', () => {
      const shortPassword = 'pass1';

      // loginSchema should accept short password
      const loginResult = loginSchema.safeParse({
        email: 'user@example.com',
        password: shortPassword,
      });
      expect(loginResult.success).toBe(true);

      // adminLoginSchema should reject short password
      const adminResult = adminLoginSchema.safeParse({
        email: 'admin@example.com',
        password: shortPassword,
      });
      expect(adminResult.success).toBe(false);
    });
  });

  describe('walletLoginSchema', () => {
    it('validates correct wallet login', () => {
      const result = walletLoginSchema.safeParse({
        address: '0x1234567890abcdef',
        chain: 'ethereum',
      });

      expect(result.success).toBe(true);
    });

    it('requires non-empty address', () => {
      const result = walletLoginSchema.safeParse({
        address: '',
        chain: 'ethereum',
      });

      expect(result.success).toBe(false);
    });

    it('requires non-empty chain', () => {
      const result = walletLoginSchema.safeParse({
        address: '0x1234567890abcdef',
        chain: '',
      });

      expect(result.success).toBe(false);
    });
  });

  describe('joinRoomSchema', () => {
    it('validates correct room code', () => {
      const result = joinRoomSchema.safeParse({ roomCode: 'ABC123' });

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.roomCode).toBe('ABC123');
      }
    });

    it('normalizes lowercase to uppercase', () => {
      const result = joinRoomSchema.safeParse({ roomCode: 'abc123' });

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.roomCode).toBe('ABC123');
      }
    });

    it('trims whitespace', () => {
      const result = joinRoomSchema.safeParse({ roomCode: '  ABC123  ' });

      expect(result.success).toBe(true);
      if (result.success) {
        expect(result.data.roomCode).toBe('ABC123');
      }
    });

    it('rejects codes not exactly 6 characters', () => {
      const result1 = joinRoomSchema.safeParse({ roomCode: 'ABC12' });
      const result2 = joinRoomSchema.safeParse({ roomCode: 'ABC1234' });

      expect(result1.success).toBe(false);
      expect(result2.success).toBe(false);
    });

    it('rejects invalid characters', () => {
      const result = joinRoomSchema.safeParse({ roomCode: 'ABC@#$' });

      expect(result.success).toBe(false);
    });
  });

  describe('inviteTokenSchema', () => {
    it('validates correct invite token', () => {
      const result = inviteTokenSchema.safeParse('validtoken123');

      expect(result.success).toBe(true);
    });

    it('enforces minimum length', () => {
      const result = inviteTokenSchema.safeParse('short');

      expect(result.success).toBe(false);
    });

    it('enforces maximum length', () => {
      const longToken = 'a'.repeat(65);
      const result = inviteTokenSchema.safeParse(longToken);

      expect(result.success).toBe(false);
    });

    it('accepts URL-safe characters', () => {
      const result = inviteTokenSchema.safeParse('valid_token-123');

      expect(result.success).toBe(true);
    });
  });

  describe('displayNameSchema', () => {
    it('validates correct display name', () => {
      const result = displayNameSchema.safeParse('Player One');

      expect(result.success).toBe(true);
    });

    it('requires non-empty name', () => {
      const result = displayNameSchema.safeParse('');

      expect(result.success).toBe(false);
    });

    it('enforces maximum length', () => {
      const longName = 'a'.repeat(33);
      const result = displayNameSchema.safeParse(longName);

      expect(result.success).toBe(false);
    });

    it('accepts special characters in names', () => {
      const result = displayNameSchema.safeParse("Jean-Pierre O'Brien");

      expect(result.success).toBe(true);
    });
  });

  describe('gameSettingsSchema', () => {
    it('validates correct game settings', () => {
      const result = gameSettingsSchema.safeParse({
        playerName: 'Player One',
        customStake: '1000',
      });

      expect(result.success).toBe(true);
    });

    it('requires non-empty player name', () => {
      const result = gameSettingsSchema.safeParse({
        playerName: '',
        customStake: '1000',
      });

      expect(result.success).toBe(false);
    });

    it('rejects negative stake', () => {
      const result = gameSettingsSchema.safeParse({
        playerName: 'Player One',
        customStake: '-100',
      });

      expect(result.success).toBe(false);
    });

    it('accepts optional custom stake', () => {
      const result = gameSettingsSchema.safeParse({
        playerName: 'Player One',
      });

      expect(result.success).toBe(true);
    });

    it('accepts positive numeric stake', () => {
      const result = gameSettingsSchema.safeParse({
        playerName: 'Player One',
        customStake: '500.50',
      });

      expect(result.success).toBe(true);
    });
  });

  describe('schema integration', () => {
    it('all schemas have safeParse method', () => {
      const schemas = [
        loginSchema,
        adminLoginSchema,
        walletLoginSchema,
        joinRoomSchema,
        inviteTokenSchema,
        displayNameSchema,
        gameSettingsSchema,
      ];

      schemas.forEach((schema) => {
        expect(typeof schema.safeParse).toBe('function');
      });
    });

    it('all schemas have parse method', () => {
      const schemas = [
        loginSchema,
        adminLoginSchema,
        walletLoginSchema,
        joinRoomSchema,
        inviteTokenSchema,
        displayNameSchema,
        gameSettingsSchema,
      ];

      schemas.forEach((schema) => {
        expect(typeof schema.parse).toBe('function');
      });
    });
  });
});
