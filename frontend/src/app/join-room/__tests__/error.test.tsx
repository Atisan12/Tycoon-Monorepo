/**
 * Tests for src/app/join-room/error.tsx
 *
 * Covers:
 *  - Rendering and layout (min-h-screen, bg-[var(--tycoon-bg)], p-4, flex centering)
 *  - reset prop type-safety: () => void (no return value expected)
 *  - digest optional on Error (Next.js route error boundary contract)
 *  - Retry / reset integration with ErrorDisplay
 *  - Home + Support navigation buttons from ErrorDisplay
 *  - Error categorization paths (network, auth, server, unknown, not-found, rate-limit)
 *  - Stale / disconnected / invalid error states
 *  - dev vs prod showTechnical visibility
 *  - ARIA: error region landmarks and accessible button labels
 *
 * Mock strategy:
 *  - useErrorReporting: stubbed so ErrorBoundary/ErrorDisplay can render without state
 *  - sanitizeError / ERROR_MESSAGES / ErrorCategory: fully controlled return values
 *  - react-i18next: key-passthrough so aria-labels are predictable
 */

import { render, screen, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';
import JoinRoomError from '../error';

// ─── Mocks ────────────────────────────────────────────────────────────────────

vi.mock('@/hooks/useErrorReporting', () => ({
  useErrorReporting: () => ({
    reportError: vi.fn(),
    clearErrors: vi.fn(),
    lastError: null,
    errorHistory: [],
  }),
}));

const mockSanitizeError = vi.fn();
vi.mock('@/lib/errors/types', () => ({
  sanitizeError: (...args: unknown[]) => mockSanitizeError(...args),
  ERROR_MESSAGES: {
    network: { title: 'Connection Issue', message: 'Check your connection.', action: 'Check Connection', supportLink: '/support/network' },
    auth: { title: 'Auth Required', message: 'Please sign in.', action: 'Sign In', supportLink: '/support/auth' },
    validation: { title: 'Invalid Input', message: 'Check your input.', action: 'Review', supportLink: '/support/validation' },
    server: { title: 'Server Error', message: 'Something went wrong on our end.', action: 'Try Again', supportLink: '/support/server' },
    not_found: { title: 'Not Found', message: 'Page not found.', action: 'Go Home', supportLink: '/support/not-found' },
    rate_limit: { title: 'Too Many Requests', message: 'Please wait and try again.', action: 'Wait & Retry', supportLink: '/support/rate-limit' },
    unknown: { title: 'Something Went Wrong', message: 'An unexpected error occurred.', action: 'Try Again', supportLink: '/support/general' },
  },
  ErrorCategory: {
    NETWORK: 'network',
    AUTH: 'auth',
    VALIDATION: 'validation',
    SERVER: 'server',
    NOT_FOUND: 'not_found',
    RATE_LIMIT: 'rate_limit',
    UNKNOWN: 'unknown',
  },
}));

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { changeLanguage: vi.fn() },
  }),
}));

// ─── Helpers ──────────────────────────────────────────────────────────────────

function makeError(message: string, digest?: string): Error & { digest?: string } {
  const err = new Error(message) as Error & { digest?: string };
  if (digest !== undefined) err.digest = digest;
  return err;
}

function makeReset(): () => void {
  return vi.fn<[], void>();
}

function defaultSanitized(overrides: Partial<{
  category: string;
  userMessage: string;
  technicalMessage: string;
  errorCode: string;
  recoverable: boolean;
  suggestedAction: string;
  supportLink: string;
}> = {}) {
  return {
    category: 'unknown',
    userMessage: 'An unexpected error occurred.',
    technicalMessage: 'Error',
    errorCode: 'TYC-UNKN-ABC123',
    recoverable: true,
    suggestedAction: 'Try Again',
    supportLink: '/support/general',
    ...overrides,
  };
}

// ─── Tests ────────────────────────────────────────────────────────────────────

describe('JoinRoomError (route error boundary)', () => {
  beforeEach(() => {
    mockSanitizeError.mockReturnValue(defaultSanitized());
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  // ── Layout & styling ────────────────────────────────────────────────────────

  describe('Layout and styling', () => {
    it('renders the outer wrapper with min-h-screen to fill the viewport', () => {
      const { container } = render(
        <JoinRoomError error={makeError('test')} reset={makeReset()} />
      );
      expect(container.querySelector('div')).toHaveClass('min-h-screen');
    });

    it('uses the tycoon theme background variable', () => {
      const { container } = render(
        <JoinRoomError error={makeError('test')} reset={makeReset()} />
      );
      expect(container.querySelector('div')).toHaveClass('bg-[var(--tycoon-bg)]');
    });

    it('centers content with flex layout', () => {
      const { container } = render(
        <JoinRoomError error={makeError('test')} reset={makeReset()} />
      );
      const wrapper = container.querySelector('div');
      expect(wrapper).toHaveClass('flex', 'items-center', 'justify-center');
    });

    it('adds p-4 padding to prevent content cutoff on small screens', () => {
      const { container } = render(
        <JoinRoomError error={makeError('test')} reset={makeReset()} />
      );
      expect(container.querySelector('div')).toHaveClass('p-4');
    });
  });

  // ── reset prop: must be () => void ──────────────────────────────────────────

  describe('reset prop — () => void contract', () => {
    it('calls reset exactly once when the retry button is clicked', () => {
      const reset = makeReset();
      render(<JoinRoomError error={makeError('test')} reset={reset} />);

      fireEvent.click(screen.getByRole('button', { name: /try again/i }));
      expect(reset).toHaveBeenCalledTimes(1);
    });

    it('does not pass a return value expectation — reset returns void', () => {
      // Asserts the type-level contract: calling reset() must not produce a value
      const reset = makeReset();
      render(<JoinRoomError error={makeError('test')} reset={reset} />);

      fireEvent.click(screen.getByRole('button', { name: /try again/i }));
      const callResult = (reset as ReturnType<typeof vi.fn>).mock.results[0];
      expect(callResult.type).toBe('return');
      expect(callResult.value).toBeUndefined();
    });

    it('can invoke reset multiple times if user clicks retry repeatedly', async () => {
      const user = userEvent.setup();
      const reset = makeReset();
      render(<JoinRoomError error={makeError('test')} reset={reset} />);

      const btn = screen.getByRole('button', { name: /try again/i });
      await user.click(btn);
      await user.click(btn);

      expect(reset).toHaveBeenCalledTimes(2);
    });
  });

  // ── digest optional ─────────────────────────────────────────────────────────

  describe('digest optional on Error', () => {
    it('renders correctly when digest is present', () => {
      const error = makeError('Crash with digest', 'abc123digest');
      render(<JoinRoomError error={error} reset={makeReset()} />);
      expect(screen.getByText(/something went wrong/i)).toBeInTheDocument();
    });

    it('renders correctly when digest is absent (undefined)', () => {
      const error = makeError('Crash without digest');
      expect(error.digest).toBeUndefined();
      render(<JoinRoomError error={error} reset={makeReset()} />);
      expect(screen.getByText(/something went wrong/i)).toBeInTheDocument();
    });

    it('renders correctly when digest is an empty string', () => {
      const error = makeError('Crash empty digest', '');
      render(<JoinRoomError error={error} reset={makeReset()} />);
      expect(screen.getByText(/something went wrong/i)).toBeInTheDocument();
    });

    it('passes the error object (with digest) through to sanitizeError', () => {
      const error = makeError('Digest pass-through', 'xyz999');
      render(<JoinRoomError error={error} reset={makeReset()} />);
      expect(mockSanitizeError).toHaveBeenCalledWith(error);
    });
  });

  // ── Error categorization ────────────────────────────────────────────────────

  describe('Error category display', () => {
    it('shows a network error title when sanitizeError returns NETWORK category', () => {
      mockSanitizeError.mockReturnValue(
        defaultSanitized({ category: 'network', userMessage: 'Check your connection.' })
      );
      render(<JoinRoomError error={makeError('network')} reset={makeReset()} />);
      expect(screen.getByText(/connection issue/i)).toBeInTheDocument();
    });

    it('shows an auth error title when sanitizeError returns AUTH category', () => {
      mockSanitizeError.mockReturnValue(
        defaultSanitized({ category: 'auth', userMessage: 'Please sign in.', recoverable: true })
      );
      render(<JoinRoomError error={makeError('auth')} reset={makeReset()} />);
      expect(screen.getByText(/auth required/i)).toBeInTheDocument();
    });

    it('shows server error title for 5xx responses', () => {
      mockSanitizeError.mockReturnValue(
        defaultSanitized({ category: 'server', userMessage: 'Something went wrong on our end.' })
      );
      render(<JoinRoomError error={makeError('500')} reset={makeReset()} />);
      expect(screen.getByText(/server error/i)).toBeInTheDocument();
    });

    it('shows not-found title for 404 errors — and still renders layout', () => {
      mockSanitizeError.mockReturnValue(
        defaultSanitized({ category: 'not_found', userMessage: 'Page not found.', recoverable: false })
      );
      const { container } = render(
        <JoinRoomError error={makeError('404')} reset={makeReset()} />
      );
      expect(screen.getByText(/not found/i)).toBeInTheDocument();
      // Layout preserved even for non-recoverable errors
      expect(container.querySelector('div')).toHaveClass('min-h-screen');
    });

    it('shows rate-limit title and retry button', () => {
      mockSanitizeError.mockReturnValue(
        defaultSanitized({ category: 'rate_limit', userMessage: 'Please wait and try again.', recoverable: true })
      );
      render(<JoinRoomError error={makeError('429')} reset={makeReset()} />);
      expect(screen.getByText(/too many requests/i)).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /wait & retry/i })).toBeInTheDocument();
    });
  });

  // ── Stale / disconnected / invalid states ───────────────────────────────────

  describe('Stale, disconnected, and invalid error states', () => {
    it('handles a stale render: same error object passed multiple times renders consistently', () => {
      const error = makeError('Stale render test');
      const reset = makeReset();
      const { rerender } = render(<JoinRoomError error={error} reset={reset} />);
      rerender(<JoinRoomError error={error} reset={reset} />);
      expect(screen.getAllByText(/something went wrong/i)).toHaveLength(1);
    });

    it('updates the display when a new error is passed (disconnected → reconnect scenario)', () => {
      const reset = makeReset();

      mockSanitizeError.mockReturnValueOnce(
        defaultSanitized({ category: 'network', userMessage: 'Check your connection.' })
      );
      const { rerender } = render(
        <JoinRoomError error={makeError('disconnect')} reset={reset} />
      );
      expect(screen.getByText(/connection issue/i)).toBeInTheDocument();

      mockSanitizeError.mockReturnValueOnce(
        defaultSanitized({ category: 'server', userMessage: 'Something went wrong on our end.' })
      );
      rerender(
        <JoinRoomError error={makeError('server crash')} reset={reset} />
      );
      expect(screen.getByText(/server error/i)).toBeInTheDocument();
    });

    it('handles an error with no message gracefully (empty string message)', () => {
      mockSanitizeError.mockReturnValue(
        defaultSanitized({ userMessage: 'An unexpected error occurred.' })
      );
      render(<JoinRoomError error={makeError('')} reset={makeReset()} />);
      expect(screen.getByText(/unexpected error/i)).toBeInTheDocument();
    });

    it('handles non-recoverable errors by hiding the retry button', () => {
      mockSanitizeError.mockReturnValue(
        defaultSanitized({ recoverable: false })
      );
      render(<JoinRoomError error={makeError('unrecoverable')} reset={makeReset()} />);
      expect(screen.queryByRole('button', { name: /try again/i })).not.toBeInTheDocument();
    });

    it('renders the Home navigation button regardless of error type', () => {
      render(<JoinRoomError error={makeError('any')} reset={makeReset()} />);
      expect(screen.getByRole('button', { name: /home/i })).toBeInTheDocument();
    });

    it('renders the Support navigation button regardless of error type', () => {
      render(<JoinRoomError error={makeError('any')} reset={makeReset()} />);
      expect(screen.getByRole('button', { name: /support/i })).toBeInTheDocument();
    });
  });

  // ── Dev vs prod: showTechnical ──────────────────────────────────────────────

  describe('showTechnical visibility (dev vs prod)', () => {
    it('does not render the technical block in production mode', () => {
      const originalEnv = process.env.NODE_ENV;
      Object.defineProperty(process.env, 'NODE_ENV', { value: 'production', configurable: true });

      mockSanitizeError.mockReturnValue(
        defaultSanitized({ technicalMessage: 'TypeError', errorCode: 'TYC-UNKN-X1' })
      );
      render(<JoinRoomError error={makeError('prod')} reset={makeReset()} />);

      // Technical block contains `Error: ...` — should NOT appear in production
      expect(screen.queryByText(/Error: TypeError/)).not.toBeInTheDocument();

      Object.defineProperty(process.env, 'NODE_ENV', { value: originalEnv, configurable: true });
    });

    it('renders the error code reference in production mode (for support)', () => {
      const originalEnv = process.env.NODE_ENV;
      Object.defineProperty(process.env, 'NODE_ENV', { value: 'production', configurable: true });

      mockSanitizeError.mockReturnValue(
        defaultSanitized({ errorCode: 'TYC-UNKN-REF99', technicalMessage: 'TypeError' })
      );
      render(<JoinRoomError error={makeError('prod-ref')} reset={makeReset()} />);

      expect(screen.getByText(/TYC-UNKN-REF99/)).toBeInTheDocument();

      Object.defineProperty(process.env, 'NODE_ENV', { value: originalEnv, configurable: true });
    });

    it('shows the technical block in development mode', () => {
      const originalEnv = process.env.NODE_ENV;
      Object.defineProperty(process.env, 'NODE_ENV', { value: 'development', configurable: true });

      mockSanitizeError.mockReturnValue(
        defaultSanitized({ technicalMessage: 'TypeError', errorCode: 'TYC-UNKN-DEV1' })
      );
      render(<JoinRoomError error={makeError('dev-error')} reset={makeReset()} />);

      // In dev, ErrorDisplay renders a technical block with `Error: <technicalMessage>`
      expect(screen.getByText(/Error: TypeError/)).toBeInTheDocument();

      Object.defineProperty(process.env, 'NODE_ENV', { value: originalEnv, configurable: true });
    });
  });

  // ── Accessibility ───────────────────────────────────────────────────────────

  describe('Accessibility', () => {
    it('has a visible heading element in the error card', () => {
      render(<JoinRoomError error={makeError('a11y test')} reset={makeReset()} />);
      expect(screen.getByRole('heading')).toBeInTheDocument();
    });

    it('retry button has an accessible label', () => {
      render(<JoinRoomError error={makeError('a11y test')} reset={makeReset()} />);
      const btn = screen.getByRole('button', { name: /try again/i });
      expect(btn).toBeInTheDocument();
    });

    it('home button is keyboard-focusable', () => {
      render(<JoinRoomError error={makeError('a11y')} reset={makeReset()} />);
      const homeBtn = screen.getByRole('button', { name: /home/i });
      homeBtn.focus();
      expect(document.activeElement).toBe(homeBtn);
    });
  });
});
