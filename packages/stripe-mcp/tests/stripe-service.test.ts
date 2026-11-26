import { describe, it, expect } from "vitest";
import {
  calculateWorkAllocation,
  createFiatPurchaseRecord,
  validateWebhookPayload,
  StripeFiatService,
  WORK_UNITS_PER_USD_CENT,
  SUPPORTED_CURRENCIES
} from "../src/stripe-service.js";

describe("StripeFiatService Unit & Invariant Tests (TDD)", () => {
  describe("calculateWorkAllocation", () => {
    it("converts USD cents to WORK units at 10 units per cent (1 USD = 1,000 WORK)", () => {
      expect(calculateWorkAllocation(1000, "usd")).toBe(10000);
      expect(calculateWorkAllocation(100, "usd")).toBe(1000);
    });

    it("supports EUR currency and normalizes case", () => {
      expect(calculateWorkAllocation(500, "EUR")).toBe(5000);
      expect(calculateWorkAllocation(500, "eur")).toBe(5000);
    });

    it("throws error when amount is zero or negative", () => {
      expect(() => calculateWorkAllocation(0, "usd")).toThrow("Amount must be greater than zero");
      expect(() => calculateWorkAllocation(-1, "usd")).toThrow("Amount must be greater than zero");
      expect(() => calculateWorkAllocation(-100, "usd")).toThrow("Amount must be greater than zero");
    });

    it("throws error for unsupported currency", () => {
      expect(() => calculateWorkAllocation(100, "jpy")).toThrow("Unsupported currency: jpy");
      expect(() => calculateWorkAllocation(100, "")).toThrow("Unsupported currency: ");
      expect(() => calculateWorkAllocation(100, "gbp")).toThrow("Unsupported currency: gbp");
    });

    it("verifies constant values", () => {
      expect(WORK_UNITS_PER_USD_CENT).toBe(10);
      expect(SUPPORTED_CURRENCIES).toEqual(["usd", "eur"]);
    });
  });

  describe("createFiatPurchaseRecord", () => {
    it("creates an idempotent purchase record with explicit timestamp and lowercase currency", () => {
      const record = createFiatPurchaseRecord({
        purchaseId: "pi_123456",
        projectId: "proj_alpha",
        amountCents: 5000,
        currency: "USD",
        timestamp: 1700000000
      });

      expect(record.purchaseId).toBe("pi_123456");
      expect(record.projectId).toBe("proj_alpha");
      expect(record.amountCents).toBe(5000);
      expect(record.currency).toBe("usd");
      expect(record.workUnits).toBe(50000);
      expect(record.status).toBe("confirmed");
      expect(record.timestamp).toBe(1700000000);
    });

    it("defaults timestamp to Date.now() when timestamp is omitted", () => {
      const before = Date.now();
      const record = createFiatPurchaseRecord({
        purchaseId: "pi_time",
        projectId: "proj_time",
        amountCents: 100,
        currency: "usd"
      });
      const after = Date.now();

      expect(record.timestamp).toBeGreaterThanOrEqual(before);
      expect(record.timestamp).toBeLessThanOrEqual(after);
    });

    it("throws when missing or whitespace purchaseId", () => {
      expect(() =>
        createFiatPurchaseRecord({
          purchaseId: "",
          projectId: "proj_alpha",
          amountCents: 1000,
          currency: "usd"
        })
      ).toThrow("missing purchaseId");

      expect(() =>
        createFiatPurchaseRecord({
          purchaseId: "   ",
          projectId: "proj_alpha",
          amountCents: 1000,
          currency: "usd"
        })
      ).toThrow("missing purchaseId");
    });

    it("throws when missing or whitespace projectId", () => {
      expect(() =>
        createFiatPurchaseRecord({
          purchaseId: "pi_123",
          projectId: "",
          amountCents: 1000,
          currency: "usd"
        })
      ).toThrow("missing projectId");

      expect(() =>
        createFiatPurchaseRecord({
          purchaseId: "pi_123",
          projectId: "   ",
          amountCents: 1000,
          currency: "usd"
        })
      ).toThrow("missing projectId");
    });
  });

  describe("validateWebhookPayload", () => {
    it("rejects non-object or null payloads", () => {
      expect(validateWebhookPayload(null)).toEqual({ valid: false, error: "Invalid payload object" });
      expect(validateWebhookPayload(undefined)).toEqual({ valid: false, error: "Invalid payload object" });
      expect(validateWebhookPayload("string")).toEqual({ valid: false, error: "Invalid payload object" });
      expect(validateWebhookPayload(123)).toEqual({ valid: false, error: "Invalid payload object" });
    });

    it("rejects non-payment_intent.succeeded event types", () => {
      const res = validateWebhookPayload({ type: "charge.succeeded" });
      expect(res.valid).toBe(false);
      expect(res.error).toBe("Unhandled event type: charge.succeeded");
    });

    it("rejects payload missing data or object or id", () => {
      expect(validateWebhookPayload({ type: "payment_intent.succeeded" })).toEqual({
        valid: false,
        error: "Missing payment intent object"
      });
      expect(validateWebhookPayload({ type: "payment_intent.succeeded", data: {} })).toEqual({
        valid: false,
        error: "Missing payment intent object"
      });
      expect(validateWebhookPayload({ type: "payment_intent.succeeded", data: { object: {} } })).toEqual({
        valid: false,
        error: "Missing payment intent object"
      });
      expect(validateWebhookPayload({ type: "payment_intent.succeeded", data: { object: { id: "" } } })).toEqual({
        valid: false,
        error: "Missing payment intent object"
      });
    });

    it("rejects payload missing or invalid projectId metadata", () => {
      const basePayload = {
        type: "payment_intent.succeeded",
        data: { object: { id: "pi_valid" } }
      };

      expect(validateWebhookPayload(basePayload)).toEqual({
        valid: false,
        error: "Missing projectId metadata in payment intent"
      });

      expect(
        validateWebhookPayload({
          ...basePayload,
          data: { object: { id: "pi_valid", metadata: { projectId: "" } } }
        })
      ).toEqual({
        valid: false,
        error: "Missing projectId metadata in payment intent"
      });

      expect(
        validateWebhookPayload({
          ...basePayload,
          data: { object: { id: "pi_valid", metadata: { projectId: "   " } } }
        })
      ).toEqual({
        valid: false,
        error: "Missing projectId metadata in payment intent"
      });

      expect(
        validateWebhookPayload({
          ...basePayload,
          data: { object: { id: "pi_valid", metadata: { projectId: 12345 } } }
        })
      ).toEqual({
        valid: false,
        error: "Missing projectId metadata in payment intent"
      });
    });

    it("validates successful payload with default usd currency", () => {
      const payload = {
        type: "payment_intent.succeeded",
        data: {
          object: {
            id: "pi_success_1",
            amount: 2000,
            metadata: { projectId: "proj_abc" }
          }
        }
      };

      const result = validateWebhookPayload(payload);
      expect(result).toEqual({
        valid: true,
        purchaseId: "pi_success_1",
        projectId: "proj_abc",
        workUnits: 20000
      });
    });

    it("validates successful payload with explicit eur currency", () => {
      const payload = {
        type: "payment_intent.succeeded",
        data: {
          object: {
            id: "pi_eur_1",
            amount: 1500,
            currency: "eur",
            metadata: { projectId: "proj_eur" }
          }
        }
      };

      const result = validateWebhookPayload(payload);
      expect(result).toEqual({
        valid: true,
        purchaseId: "pi_eur_1",
        projectId: "proj_eur",
        workUnits: 15000
      });
    });

    it("catches calculation errors (e.g. negative amount or unsupported currency) and reports error", () => {
      const payloadBadAmount = {
        type: "payment_intent.succeeded",
        data: {
          object: {
            id: "pi_neg",
            amount: -50,
            currency: "usd",
            metadata: { projectId: "proj_neg" }
          }
        }
      };

      const resultBadAmount = validateWebhookPayload(payloadBadAmount);
      expect(resultBadAmount.valid).toBe(false);
      expect(resultBadAmount.error).toBe("Amount must be greater than zero");

      const payloadBadCurr = {
        type: "payment_intent.succeeded",
        data: {
          object: {
            id: "pi_curr",
            amount: 100,
            currency: "bitcoin",
            metadata: { projectId: "proj_curr" }
          }
        }
      };

      const resultBadCurr = validateWebhookPayload(payloadBadCurr);
      expect(resultBadCurr.valid).toBe(false);
      expect(resultBadCurr.error).toBe("Unsupported currency: bitcoin");
    });
  });

  describe("StripeFiatService Class & Options", () => {
    it("instantiates with default options and processes funding", () => {
      const defaultService = new StripeFiatService();
      expect(defaultService.getConversionRate()).toBe(10);
      expect(defaultService.getSupportedCurrencies()).toEqual(["usd", "eur"]);

      const receipt = defaultService.processFunding({
        purchaseId: "pi_def",
        projectId: "proj_def",
        amountCents: 1000,
        currency: "usd"
      });
      expect(receipt.workUnits).toBe(10000);
    });

    it("instantiates with custom options", () => {
      const customService = new StripeFiatService({
        conversionRatePerCent: 20,
        supportedCurrencies: ["usd", "eur", "gbp"]
      });
      expect(customService.getConversionRate()).toBe(20);
      expect(customService.getSupportedCurrencies()).toEqual(["usd", "eur", "gbp"]);

      const receipt = customService.processFunding({
        purchaseId: "pi_custom",
        projectId: "proj_custom",
        amountCents: 1000,
        currency: "usd"
      });
      expect(receipt.workUnits).toBe(20000);
    });
  });
});
