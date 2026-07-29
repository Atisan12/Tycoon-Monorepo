import React from "react";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { vi, describe, it, expect, beforeEach } from "vitest";
import {
  NearWalletProvider,
  useNearWallet,
} from "../near-wallet-provider";

// ── Mocks ─────────────────────────────────────────────────────────────────────

vi.mock("react-toastify", () => ({
  toast: {
    error: vi.fn(),
  },
}));

vi.mock("@/lib/near", () => ({
  getNearNetworkId: () => "testnet",
  getNearContractId: () => "tycoon.testnet",
  nearErrorMessage: (e: unknown) => String(e),
  isValidNearAccountId: (id: string) => id.includes("."),
  isLikelyUserRejectedError: (e: unknown) => String(e).includes("rejected"),
  sanitizeErrorMessage: (msg: string) => msg,
  NEAR_SIGNATURE_REJECTED_MESSAGE: "User rejected the transaction",
  getTransactionHashFromOutcome: () => "abc123",
  isFinalExecutionSuccess: () => true,
  getExplorerTransactionUrl: () => "https://explorer.near.org",
  isDepositSafe: () => true,
  DEFAULT_FUNCTION_CALL_GAS: BigInt(30000000000000),
  MAX_DEPOSIT_YOCTO: BigInt(1000000000000000000),
  trackNearWalletConnected: vi.fn(),
  trackNearWalletDisconnected: vi.fn(),
  trackNearTxSubmitted: vi.fn(),
  trackNearTxConfirmed: vi.fn(),
  trackNearTxFailed: vi.fn(),
}));

// ── Tests ─────────────────────────────────────────────────────────────────────

describe("NearWalletProvider", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  function TestConsumer() {
    const {
      ready,
      initError,
      connectError,
      disconnectError,
      accountId,
      connect,
      disconnect,
      clearError,
    } = useNearWallet();

    return (
      <div>
        <span data-testid="ready">{String(ready)}</span>
        <span data-testid="init-error">{initError ?? "null"}</span>
        <span data-testid="connect-error">{connectError ?? "null"}</span>
        <span data-testid="disconnect-error">{disconnectError ?? "null"}</span>
        <span data-testid="account-id">{accountId ?? "null"}</span>
        <button onClick={connect} data-testid="connect-btn">
          Connect
        </button>
        <button onClick={disconnect} data-testid="disconnect-btn">
          Disconnect
        </button>
        <button onClick={clearError} data-testid="clear-error-btn">
          Clear Error
        </button>
      </div>
    );
  }

  it("exposes ready, initError, connectError, and disconnectError states", async () => {
    render(
      <NearWalletProvider>
        <TestConsumer />
      </NearWalletProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId("ready").textContent).toBe("true");
    });

    expect(screen.getByTestId("init-error").textContent).toBe("null");
    expect(screen.getByTestId("connect-error").textContent).toBe("null");
    expect(screen.getByTestId("disconnect-error").textContent).toBe("null");
  });

  it("exposes clearError function to clear all error states", async () => {
    render(
      <NearWalletProvider>
        <TestConsumer />
      </NearWalletProvider>
    );

    await waitFor(() => {
      expect(screen.getByTestId("ready").textContent).toBe("true");
    });

    const clearErrorBtn = screen.getByTestId("clear-error-btn");
    await userEvent.click(clearErrorBtn);

    expect(screen.getByTestId("connect-error").textContent).toBe("null");
    expect(screen.getByTestId("disconnect-error").textContent).toBe("null");
  });

  it("throws when useNearWallet is used outside provider", () => {
    function TestComponent() {
      useNearWallet();
      return null;
    }

    // Suppress console.error for this test
    const consoleSpy = vi.spyOn(console, "error").mockImplementation(() => {});

    expect(() => {
      render(<TestComponent />);
    }).toThrow("useNearWallet must be used within NearWalletProvider");

    consoleSpy.mockRestore();
  });
});
