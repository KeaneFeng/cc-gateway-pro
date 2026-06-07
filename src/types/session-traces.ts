export type SessionTraceMode = "off" | "summary" | "full";

export interface SessionTraceSettings {
  enabled: boolean;
  mode: SessionTraceMode;
  retentionDays: number;
  maxResponseTextChars: number;
  captureRequestJson: boolean;
  captureResponseJson: boolean;
  redactSensitiveValues: boolean;
}

export interface TraceSessionFilters {
  appType?: string;
  search?: string;
  limit?: number;
}

export interface TraceSessionSummary {
  sessionId: string;
  title?: string | null;
  appType: string;
  providerId?: string | null;
  model?: string | null;
  turnCount: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCacheReadTokens: number;
  totalCacheCreationTokens: number;
  totalCostUsd: string;
  avgLatencyMs?: number | null;
  lastStatusCode?: number | null;
  firstSeenAt: number;
  lastSeenAt: number;
}

export interface TraceSessionDetailRequest {
  sessionId: string;
  appType?: string;
}

export interface TraceTurnDetail {
  traceId: string;
  turnIndex?: number | null;
  providerId?: string | null;
  model?: string | null;
  requestModel?: string | null;
  isStreaming: boolean;
  statusCode?: number | null;
  systemPromptPreview?: string | null;
  systemPromptHash?: string | null;
  messageCount: number;
  toolCount: number;
  requestSummary: Record<string, unknown>;
  contextStats: {
    totalTokens?: number;
    categories?: Record<string, number>;
    resources?: Record<
      string,
      {
        title?: string;
        count?: number;
        totalTokens?: number;
        groups?: Record<
          string,
          Array<{
            name?: string;
            tokens?: number;
            tokenLabel?: string;
            source?: string;
          }>
        >;
      }
    >;
    contextCommand?: {
      model?: string | null;
      usedTokens?: number | null;
      windowTokens?: number | null;
      usageRatio?: number | null;
      categories?: Record<
        string,
        {
          tokens?: number;
          ratio?: number | null;
        }
      >;
    };
    estimator?: string;
    [key: string]: unknown;
  };
  contextWindowTokens?: number | null;
  contextUsedTokens?: number | null;
  contextUsageRatio?: number | null;
  responseTextPreview?: string | null;
  toolCalls: unknown[];
  stopReason?: string | null;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheCreationTokens: number;
  latencyMs?: number | null;
  firstTokenMs?: number | null;
  traceMode: SessionTraceMode;
  createdAt: number;
}

export interface TraceSessionDetail {
  sessionId: string;
  title?: string | null;
  appType?: string | null;
  turnCount: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalCacheReadTokens: number;
  totalCacheCreationTokens: number;
  avgContextUsageRatio?: number | null;
  maxContextUsedTokens?: number | null;
  traces: TraceTurnDetail[];
}
