/**
 * Gate 9 / Fiat Onramp Service: Stripe MCP helper
 * Implements deterministic conversion and receipt tracking between Fiat (USD/EUR) and WORK tokens.
 * In Gate 5/9: 1 simulated stablecoin (1.00 USD) = 1,000 WORK units.
 * 1 USD cent = 10 WORK units.
 */

export const WORK_UNITS_PER_USD_CENT = 10;
export const SUPPORTED_CURRENCIES = ["usd", "eur"];

export interface FiatPurchaseParams {
  purchaseId: string;
  projectId: string;
  amountCents: number;
  currency: string;
  timestamp?: number;
}

export interface FiatPurchaseRecord {
  purchaseId: string;
  projectId: string;
  amountCents: number;
  currency: string;
  workUnits: number;
  status: "confirmed" | "failed";
  timestamp: number;
}

export interface WebhookValidationResult {
  valid: boolean;
  purchaseId?: string;
  projectId?: string;
  workUnits?: number;
  error?: string;
}

export function calculateWorkAllocation(
  amountCents: number,
  currency: string,
  rate: number = WORK_UNITS_PER_USD_CENT,
  supported: string[] = SUPPORTED_CURRENCIES
): number {
  if (amountCents <= 0) {
    throw new Error("Amount must be greater than zero");
  }
  const normalizedCurrency = currency.toLowerCase();
  if (!supported.includes(normalizedCurrency)) {
    throw new Error(`Unsupported currency: ${currency}`);
  }
  return amountCents * rate;
}

export function createFiatPurchaseRecord(
  params: FiatPurchaseParams,
  rate: number = WORK_UNITS_PER_USD_CENT,
  supported: string[] = SUPPORTED_CURRENCIES
): FiatPurchaseRecord {
  if (!params.purchaseId || params.purchaseId.trim() === "") {
    throw new Error("Invalid purchase record parameters: missing purchaseId");
  }
  if (!params.projectId || params.projectId.trim() === "") {
    throw new Error("Invalid purchase record parameters: missing projectId");
  }

  const workUnits = calculateWorkAllocation(params.amountCents, params.currency, rate, supported);
  return {
    purchaseId: params.purchaseId,
    projectId: params.projectId,
    amountCents: params.amountCents,
    currency: params.currency.toLowerCase(),
    workUnits,
    status: "confirmed",
    timestamp: params.timestamp ?? Date.now()
  };
}

export function validateWebhookPayload(payload: any): WebhookValidationResult {
  if (!payload || typeof payload !== "object") {
    return { valid: false, error: "Invalid payload object" };
  }

  if (payload.type !== "payment_intent.succeeded") {
    return { valid: false, error: `Unhandled event type: ${payload.type}` };
  }

  const obj = payload.data?.object;
  if (!obj || !obj.id) {
    return { valid: false, error: "Missing payment intent object" };
  }

  const projectId = obj.metadata?.projectId;
  if (!projectId || typeof projectId !== "string" || projectId.trim() === "") {
    return { valid: false, error: "Missing projectId metadata in payment intent" };
  }

  try {
    const currency = obj.currency && typeof obj.currency === "string" ? obj.currency : "usd";
    const workUnits = calculateWorkAllocation(obj.amount, currency);
    return {
      valid: true,
      purchaseId: obj.id,
      projectId,
      workUnits
    };
  } catch (err: any) {
    return { valid: false, error: err.message };
  }
}

export class StripeFiatService {
  private conversionRate: number;
  private supportedCurrencies: string[];

  constructor(options?: { conversionRatePerCent?: number; supportedCurrencies?: string[] }) {
    this.conversionRate = options?.conversionRatePerCent ?? WORK_UNITS_PER_USD_CENT;
    this.supportedCurrencies = options?.supportedCurrencies ?? SUPPORTED_CURRENCIES;
  }

  public getConversionRate(): number {
    return this.conversionRate;
  }

  public getSupportedCurrencies(): string[] {
    return this.supportedCurrencies;
  }

  public processFunding(params: FiatPurchaseParams): FiatPurchaseRecord {
    return createFiatPurchaseRecord(params, this.conversionRate, this.supportedCurrencies);
  }
}
