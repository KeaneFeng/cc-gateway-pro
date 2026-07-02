import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  Activity,
  AlertTriangle,
  BarChart2,
  Clock,
  Database,
  FileSearch,
  Hash,
  MessageSquare,
  Plug,
  RefreshCw,
  Search,
  Settings,
  ShieldOff,
} from "lucide-react";
import { useQueryClient } from "@tanstack/react-query";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ProviderIcon } from "@/components/ProviderIcon";
import {
  useSessionTraceSettingsQuery,
  useUpdateSessionTraceSettingsMutation,
  useTraceSessionDetailQuery,
  useTraceSessionsQuery,
} from "@/lib/query/session-traces";
import type {
  SessionTraceMode,
  SessionTraceSettings,
  TraceSessionDetail,
  TraceSessionSummary,
  TraceTurnDetail,
} from "@/types/session-traces";
import { cn } from "@/lib/utils";
import {
  formatTimestamp,
  getProviderIconName,
} from "@/components/sessions/utils";

export interface SessionTraceTarget {
  sessionId: string;
  appType?: string;
}

const DEFAULT_TRACE_SETTINGS: SessionTraceSettings = {
  enabled: false,
  mode: "off",
  retentionDays: 14,
  maxResponseTextChars: 2000,
  captureRequestJson: false,
  captureResponseJson: false,
  redactSensitiveValues: true,
};

const PREFERRED_AGENT_ORDER = [
  "all",
  "claude",
  "claude-desktop",
  "codex",
  "hermes",
  "gemini",
  "opencode",
  "openclaw",
];

const RESOURCE_DISPLAY_ORDER = [
  "usedSkills",
  "skills",
  "mcpTools",
  "customAgents",
  "agentTools",
  "memoryFiles",
  "plugins",
  "tools",
];

const MEANINGFUL_TRACE_RESOURCES = new Set([
  "usedSkills",
  "skills",
  "mcpTools",
  "customAgents",
  "agentTools",
  "memoryFiles",
  "plugins",
]);

function formatTokens(value: number) {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}k`;
  return value.toLocaleString();
}

function parseCost(value: string) {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? parsed : 0;
}

function formatUsd(value: string) {
  return `$${parseCost(value).toFixed(4)}`;
}

function formatRatio(value?: number | null) {
  if (value == null || !Number.isFinite(value)) return "--";
  return `${Math.max(0, value * 100).toFixed(1)}%`;
}

type ResourceGroups = NonNullable<
  NonNullable<TraceTurnDetail["contextStats"]["resources"]>[string]["groups"]
>;

function getResourceTotals(trace?: TraceTurnDetail | null) {
  const resources = trace?.contextStats.resources ?? {};
  return Object.entries(resources)
    .map(([key, resource]) => ({
      key,
      title: resource.title || key,
      count: resource.count ?? 0,
      totalTokens: resource.totalTokens ?? 0,
      groups: resource.groups ?? {},
    }))
    .filter((resource) => resource.count > 0)
    .sort((a, b) => {
      const orderA = RESOURCE_DISPLAY_ORDER.indexOf(a.key);
      const orderB = RESOURCE_DISPLAY_ORDER.indexOf(b.key);
      return (
        (orderA === -1 ? Number.MAX_SAFE_INTEGER : orderA) -
          (orderB === -1 ? Number.MAX_SAFE_INTEGER : orderB) ||
        a.title.localeCompare(b.title)
      );
    });
}

function getLocalContextEnrichment(trace?: TraceTurnDetail | null) {
  const enrichment = trace?.contextStats.localContextEnrichment;
  if (!enrichment) return null;
  const status = enrichment?.status;
  if (status !== "found" && status !== "missing") return null;
  return {
    status,
    source: enrichment.source,
  };
}

function topResourceItems(groups: ResourceGroups, limit = 4) {
  return Object.values(groups)
    .flat()
    .filter((item) => item.name)
    .sort((a, b) => (b.tokens ?? 0) - (a.tokens ?? 0))
    .slice(0, limit);
}

function getQualityTone(session: TraceSessionSummary) {
  const total =
    session.totalInputTokens +
    session.totalOutputTokens +
    session.totalCacheReadTokens +
    session.totalCacheCreationTokens;
  if ((session.lastStatusCode ?? 200) >= 400) {
    return {
      label: "error",
      className: "bg-red-100 text-red-700 dark:bg-red-950/50 dark:text-red-300",
    };
  }
  if (total > 500_000) {
    return {
      label: "watch",
      className:
        "bg-amber-100 text-amber-700 dark:bg-amber-950/50 dark:text-amber-300",
    };
  }
  return {
    label: "healthy",
    className:
      "bg-emerald-100 text-emerald-700 dark:bg-emerald-950/50 dark:text-emerald-300",
  };
}

function getTraceSessionTitle(
  session?: TraceSessionSummary | null,
  detail?: TraceSessionDetail,
  t?: ReturnType<typeof useTranslation>["t"],
) {
  const title = detail?.title || session?.title;
  if (title?.trim()) return title.trim();
  if (!session) return "";
  const parts = [
    t ? getAppLabel(session.appType, t) : session.appType,
    session.model,
    session.lastSeenAt
      ? new Date(session.lastSeenAt * 1000).toLocaleString()
      : null,
  ].filter(Boolean);
  return parts.join(" · ") || session.sessionId.slice(0, 8);
}

function getAppLabel(
  appType: string,
  t: ReturnType<typeof useTranslation>["t"],
) {
  if (appType === "all") {
    return t("sessionTraces.allAgents", { defaultValue: "All" });
  }
  return t(`apps.${appType}`, { defaultValue: appType });
}

interface TraceSessionItemProps {
  session: TraceSessionSummary;
  selected: boolean;
  onSelect: (sessionId: string) => void;
}

function TraceSessionItem({
  session,
  selected,
  onSelect,
}: TraceSessionItemProps) {
  const { t } = useTranslation();
  const totalTokens =
    session.totalInputTokens +
    session.totalOutputTokens +
    session.totalCacheReadTokens +
    session.totalCacheCreationTokens;
  const tone = getQualityTone(session);
  const title = getTraceSessionTitle(session, undefined, t);

  return (
    <button
      type="button"
      onClick={() => onSelect(session.sessionId)}
      className={cn(
        "w-full rounded-lg border px-3 py-2.5 text-left transition-colors",
        selected
          ? "border-primary/40 bg-primary/10"
          : "border-transparent hover:bg-muted/60",
      )}
    >
      <div className="flex items-center gap-2">
        <ProviderIcon
          icon={getProviderIconName(session.appType)}
          name={session.appType}
          size={18}
        />
        <div className="min-w-0 flex-1">
          <div className="line-clamp-2 text-sm font-medium leading-snug">
            {title}
          </div>
          <div className="mt-0.5 truncate text-xs text-muted-foreground">
            {session.model || session.appType}
            {` · ${session.sessionId.slice(0, 8)}`}
          </div>
        </div>
        <Badge className={cn("shrink-0 text-[10px]", tone.className)}>
          {tone.label}
        </Badge>
      </div>
      <div className="mt-2 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-muted-foreground">
        <span>{session.turnCount} turns</span>
        <span>{formatTokens(totalTokens)} tokens</span>
        <span>{formatUsd(session.totalCostUsd)}</span>
      </div>
    </button>
  );
}

interface EmptyStateProps {
  enabled?: boolean;
}

function EmptyState({ enabled }: EmptyStateProps) {
  const { t } = useTranslation();
  return (
    <div className="flex h-full flex-col items-center justify-center p-8 text-center text-muted-foreground">
      {enabled ? (
        <FileSearch className="mb-3 size-10 opacity-40" />
      ) : (
        <ShieldOff className="mb-3 size-10 opacity-40" />
      )}
      <h3 className="text-sm font-medium text-foreground">
        {enabled
          ? t("sessionTraces.noSessionTraces", {
              defaultValue: "暂无 Session Traces",
            })
          : t("sessionTraces.collectionOff", {
              defaultValue: "Session Traces 已关闭",
            })}
      </h3>
      <p className="mt-1 max-w-md text-sm">
        {enabled
          ? t("sessionTraces.noSessionTracesDescription", {
              defaultValue:
                "开启采集后，新请求会出现在这里；历史会话只会显示已有 usage 聚合。",
            })
          : t("sessionTraces.collectionOffDescription", {
              defaultValue:
                "现有历史统计仍可查看，但新的上下文 trace 不会被记录。可在高级设置中开启 Summary 模式。",
            })}
      </p>
    </div>
  );
}

function Metric({
  label,
  value,
  icon: Icon,
}: {
  label: string;
  value: string;
  icon: typeof Activity;
}) {
  return (
    <div className="rounded-lg border bg-card/50 p-3">
      <div className="flex items-center gap-2 text-xs text-muted-foreground">
        <Icon className="size-3.5" />
        {label}
      </div>
      <div className="mt-1 text-lg font-semibold">{value}</div>
    </div>
  );
}

function ResourceInventory({
  trace,
  compact = false,
  includeGenericTools = false,
}: {
  trace?: TraceTurnDetail | null;
  compact?: boolean;
  includeGenericTools?: boolean;
}) {
  const resources = getResourceTotals(trace).filter(
    (resource) => includeGenericTools || resource.key !== "tools",
  );
  if (resources.length === 0) return null;

  return (
    <div
      className={cn(
        "grid gap-3",
        compact ? "md:grid-cols-2" : "lg:grid-cols-2",
      )}
    >
      {resources.map((resource) => {
        const topItems = topResourceItems(resource.groups, compact ? 3 : 8);
        return (
          <Card key={resource.key} className="bg-card/50">
            <CardHeader className="py-3">
              <CardTitle className="flex items-center justify-between gap-2 text-sm">
                <span className="flex min-w-0 items-center gap-2">
                  <Plug className="size-3.5 shrink-0 text-muted-foreground" />
                  <span className="truncate">{resource.title}</span>
                </span>
                <Badge variant="secondary" className="shrink-0">
                  {resource.count}
                </Badge>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-3 text-xs">
              <div className="flex justify-between text-muted-foreground">
                <span>Total tokens</span>
                <span className="font-mono">
                  {resource.totalTokens > 0
                    ? formatTokens(resource.totalTokens)
                    : "--"}
                </span>
              </div>
              {Object.entries(resource.groups).map(([group, items]) => (
                <div key={group} className="space-y-1">
                  <div className="font-medium text-muted-foreground">
                    {group}
                  </div>
                  <div className="space-y-1">
                    {(compact ? items.slice(0, 4) : items).map((item) => (
                      <div
                        key={`${group}:${item.name}`}
                        className="flex justify-between gap-2 rounded-md bg-muted/35 px-2 py-1"
                      >
                        <span className="min-w-0 truncate">{item.name}</span>
                        <span className="shrink-0 font-mono text-muted-foreground">
                          {item.tokenLabel || formatTokens(item.tokens ?? 0)}
                        </span>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
              {compact && topItems.length > 0 && (
                <div className="flex flex-wrap gap-1.5 pt-1">
                  {topItems.map((item) => (
                    <Badge
                      key={item.name}
                      variant="outline"
                      className="max-w-full truncate"
                    >
                      {item.name}
                    </Badge>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}

function ResourceTags({ trace }: { trace: TraceTurnDetail }) {
  const resources = getResourceTotals(trace).filter((resource) =>
    MEANINGFUL_TRACE_RESOURCES.has(resource.key),
  );
  const groupedTags = resources
    .map((resource) => {
      const topItems = topResourceItems(resource.groups, 6);
      const tags =
        topItems.length === 0
          ? [
              {
                key: resource.key,
                label: resource.title,
                title: `${resource.title}: ${resource.count}`,
              },
            ]
          : topItems.map((item) => ({
              key: `${resource.key}:${item.name}`,
              label: item.name || resource.title,
              title: `${resource.title} · ${item.name}${item.tokenLabel ? ` · ${item.tokenLabel}` : ""}`,
            }));
      return {
        key: resource.key,
        title: resource.title,
        tags,
      };
    })
    .filter((group) => group.tags.length > 0);

  if (groupedTags.length === 0) return null;

  return (
    <div className="mt-3 space-y-2">
      {groupedTags.slice(0, 4).map((group) => (
        <div
          key={group.key}
          className="grid gap-1.5 text-xs sm:grid-cols-[84px_minmax(0,1fr)]"
        >
          <div className="font-medium text-muted-foreground">{group.title}</div>
          <div className="flex min-w-0 flex-wrap gap-1.5">
            {group.tags.slice(0, 8).map((tag) => (
              <Badge
                key={tag.key}
                variant="outline"
                className="max-w-full truncate text-[11px]"
                title={tag.title}
              >
                {tag.label}
              </Badge>
            ))}
          </div>
        </div>
      ))}
    </div>
  );
}

function LocalContextEnrichmentNotice({
  trace,
}: {
  trace?: TraceTurnDetail | null;
}) {
  const { t } = useTranslation();
  const enrichment = getLocalContextEnrichment(trace);
  if (!enrichment) return null;

  const isFound = enrichment.status === "found";
  return (
    <div
      className={cn(
        "flex items-start gap-2 rounded-md border px-3 py-2 text-xs",
        isFound
          ? "border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900/60 dark:bg-emerald-950/30 dark:text-emerald-200"
          : "border-amber-200 bg-amber-50 text-amber-800 dark:border-amber-900/60 dark:bg-amber-950/30 dark:text-amber-200",
      )}
    >
      {isFound ? (
        <Database className="mt-0.5 size-3.5 shrink-0" />
      ) : (
        <FileSearch className="mt-0.5 size-3.5 shrink-0" />
      )}
      <span>
        {isFound
          ? t("sessionTraces.contextEnrichmentCached", {
              defaultValue:
                "Claude /context data has been saved to the trace database and will be reused without rescanning.",
            })
          : t("sessionTraces.contextEnrichmentMissing", {
              defaultValue:
                "No local Claude /context output was found for this session; this result is cached to avoid repeated scans.",
            })}
      </span>
    </div>
  );
}

function SessionOverview({
  session,
  detail,
}: {
  session: TraceSessionSummary;
  detail?: TraceSessionDetail;
}) {
  const { t } = useTranslation();
  const totalTokens =
    session.totalInputTokens +
    session.totalOutputTokens +
    session.totalCacheReadTokens +
    session.totalCacheCreationTokens;
  const latestTrace = detail?.traces[0];

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-4">
        <Metric
          icon={Activity}
          label={t("sessionTraces.turns", { defaultValue: "Turns" })}
          value={session.turnCount.toLocaleString()}
        />
        <Metric
          icon={Database}
          label={t("sessionTraces.tokens", { defaultValue: "Tokens" })}
          value={formatTokens(totalTokens)}
        />
        <Metric
          icon={BarChart2}
          label={t("sessionTraces.cost", { defaultValue: "Cost" })}
          value={formatUsd(session.totalCostUsd)}
        />
        <Metric
          icon={Clock}
          label={t("sessionTraces.avgLatency", {
            defaultValue: "Avg latency",
          })}
          value={
            session.avgLatencyMs == null ? "--" : `${session.avgLatencyMs}ms`
          }
        />
      </div>

      {latestTrace && (
        <div className="grid gap-3 md:grid-cols-4">
          <Metric
            icon={BarChart2}
            label={t("sessionTraces.contextRatio", {
              defaultValue: "Context ratio",
            })}
            value={formatRatio(latestTrace.contextUsageRatio)}
          />
          <Metric
            icon={Plug}
            label={t("sessionTraces.mcpTools", {
              defaultValue: "MCP tools",
            })}
            value={String(
              latestTrace.contextStats.resources?.mcpTools?.count ?? 0,
            )}
          />
          <Metric
            icon={Settings}
            label={t("sessionTraces.skills", {
              defaultValue: "Skills",
            })}
            value={String(
              latestTrace.contextStats.resources?.skills?.count ?? 0,
            )}
          />
          <Metric
            icon={MessageSquare}
            label={t("sessionTraces.memoryAgents", {
              defaultValue: "Agents/memory",
            })}
            value={String(
              (latestTrace.contextStats.resources?.memoryFiles?.count ?? 0) +
                (latestTrace.contextStats.resources?.customAgents?.count ?? 0) +
                (latestTrace.contextStats.resources?.agentTools?.count ?? 0),
            )}
          />
        </div>
      )}

      <div className="grid gap-4 lg:grid-cols-2">
        <Card>
          <CardHeader className="py-3">
            <CardTitle className="text-sm">
              {t("sessionTraces.usageBreakdown", {
                defaultValue: "Usage Breakdown",
              })}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Input</span>
              <span className="font-mono">
                {formatTokens(session.totalInputTokens)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Output</span>
              <span className="font-mono">
                {formatTokens(session.totalOutputTokens)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Cache read</span>
              <span className="font-mono">
                {formatTokens(session.totalCacheReadTokens)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Cache write</span>
              <span className="font-mono">
                {formatTokens(session.totalCacheCreationTokens)}
              </span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="py-3">
            <CardTitle className="text-sm">
              {t("sessionTraces.qualitySignals", {
                defaultValue: "Quality Signals",
              })}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Last status</span>
              <span className="font-mono">
                {session.lastStatusCode ?? "--"}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">First seen</span>
              <span>{formatTimestamp(session.firstSeenAt * 1000)}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Last seen</span>
              <span>{formatTimestamp(session.lastSeenAt * 1000)}</span>
            </div>
            <div className="flex items-start gap-2 rounded-md bg-muted/50 p-2 text-xs text-muted-foreground">
              <AlertTriangle className="mt-0.5 size-3.5 shrink-0" />
              <span>
                {t("sessionTraces.estimatedHint", {
                  defaultValue:
                    "当前为 usage 聚合基础视图；开启 Session Traces 后会显示 context 分类、工具调用和请求摘要。",
                })}
              </span>
            </div>
          </CardContent>
        </Card>
      </div>

      <ResourceInventory trace={latestTrace} compact />
    </div>
  );
}

function ContextBreakdown({
  detail,
  loading,
  enabled,
}: {
  detail?: TraceSessionDetail;
  loading: boolean;
  enabled: boolean;
}) {
  const { t } = useTranslation();
  const latestTrace = detail?.traces[0];
  const categories = latestTrace?.contextStats.categories ?? {};
  const contextCommand = latestTrace?.contextStats.contextCommand;
  const commandCategories = contextCommand?.categories ?? {};
  const total =
    contextCommand?.usedTokens ?? latestTrace?.contextStats.totalTokens ?? 0;
  const rows = Object.entries(categories).filter(([, value]) => value > 0);
  const commandRows = Object.entries(commandCategories).filter(
    ([, value]) => (value.tokens ?? 0) > 0,
  );

  if (loading) {
    return (
      <div className="flex justify-center py-10">
        <RefreshCw className="size-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!detail || detail.traces.length === 0) {
    return <EmptyState enabled={enabled} />;
  }

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-3">
        <Metric
          icon={Database}
          label={t("sessionTraces.latestContext", {
            defaultValue: "Latest context",
          })}
          value={formatTokens(
            contextCommand?.usedTokens ??
              latestTrace?.contextUsedTokens ??
              total,
          )}
        />
        <Metric
          icon={BarChart2}
          label={t("sessionTraces.contextRatio", {
            defaultValue: "Context ratio",
          })}
          value={formatRatio(
            contextCommand?.usageRatio ?? latestTrace?.contextUsageRatio,
          )}
        />
        <Metric
          icon={Activity}
          label={t("sessionTraces.maxContext", {
            defaultValue: "Max context",
          })}
          value={formatTokens(detail.maxContextUsedTokens ?? 0)}
        />
      </div>

      <LocalContextEnrichmentNotice trace={latestTrace} />

      <Card>
        <CardHeader className="py-3">
          <CardTitle className="text-sm">
            {t("sessionTraces.latestContextBreakdown", {
              defaultValue: "Latest Context Breakdown",
            })}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          {rows.length === 0 ? (
            <div className="text-sm text-muted-foreground">
              {t("sessionTraces.noContextBreakdown", {
                defaultValue: "No context category data for this turn.",
              })}
            </div>
          ) : (
            rows.map(([name, value]) => {
              const percent =
                total > 0 ? Math.min(100, (value / total) * 100) : 0;
              return (
                <div key={name} className="space-y-1.5">
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">{name}</span>
                    <span className="font-mono">{formatTokens(value)}</span>
                  </div>
                  <div className="h-2 overflow-hidden rounded-full bg-muted">
                    <div
                      className="h-full rounded-full bg-primary"
                      style={{ width: `${percent}%` }}
                    />
                  </div>
                </div>
              );
            })
          )}
        </CardContent>
      </Card>

      {commandRows.length > 0 && (
        <Card>
          <CardHeader className="py-3">
            <CardTitle className="text-sm">
              {t("sessionTraces.claudeContextBreakdown", {
                defaultValue: "Claude Context Breakdown",
              })}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            {commandRows.map(([name, value]) => (
              <div key={name} className="flex justify-between text-sm">
                <span className="text-muted-foreground">{name}</span>
                <span className="font-mono">
                  {formatTokens(value.tokens ?? 0)}
                  {value.ratio != null ? ` · ${formatRatio(value.ratio)}` : ""}
                </span>
              </div>
            ))}
          </CardContent>
        </Card>
      )}

      <ResourceInventory trace={latestTrace} />
    </div>
  );
}

function TraceTurnCard({ trace }: { trace: TraceTurnDetail }) {
  const totalTokens =
    trace.inputTokens +
    trace.outputTokens +
    trace.cacheReadTokens +
    trace.cacheCreationTokens;

  return (
    <div className="rounded-lg border bg-card/50 p-3">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <Badge variant="secondary">#{trace.turnIndex ?? "-"}</Badge>
            <span className="truncate text-sm font-medium">
              {trace.model || trace.requestModel || "unknown"}
            </span>
            <Badge variant={trace.isStreaming ? "outline" : "secondary"}>
              {trace.isStreaming ? "stream" : "json"}
            </Badge>
          </div>
          <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
            <span>{formatTimestamp(trace.createdAt * 1000)}</span>
            <span>{formatTokens(totalTokens)} tokens</span>
            {trace.latencyMs != null && <span>{trace.latencyMs}ms</span>}
            {trace.stopReason && <span>{trace.stopReason}</span>}
          </div>
        </div>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <MessageSquare className="size-3.5" />
          <span>{trace.messageCount}</span>
        </div>
      </div>

      <div className="mt-3 grid gap-2 text-xs md:grid-cols-3">
        <div className="rounded-md bg-muted/40 p-2">
          <div className="text-muted-foreground">Input</div>
          <div className="font-mono">{formatTokens(trace.inputTokens)}</div>
        </div>
        <div className="rounded-md bg-muted/40 p-2">
          <div className="text-muted-foreground">Output</div>
          <div className="font-mono">{formatTokens(trace.outputTokens)}</div>
        </div>
        <div className="rounded-md bg-muted/40 p-2">
          <div className="text-muted-foreground">Context</div>
          <div className="font-mono">
            {formatTokens(
              trace.contextUsedTokens ?? trace.contextStats.totalTokens ?? 0,
            )}
          </div>
        </div>
      </div>

      {trace.systemPromptHash && (
        <div className="mt-3 flex items-center gap-2 truncate rounded-md bg-muted/30 px-2 py-1.5 text-xs text-muted-foreground">
          <Hash className="size-3.5 shrink-0" />
          <span className="truncate">{trace.systemPromptHash}</span>
        </div>
      )}

      {trace.responseTextPreview && (
        <div className="mt-3 whitespace-pre-wrap rounded-md bg-muted/30 p-2 text-xs leading-relaxed text-muted-foreground">
          {trace.responseTextPreview}
        </div>
      )}

      <ResourceTags trace={trace} />
    </div>
  );
}

function TraceTurns({
  detail,
  loading,
  enabled,
}: {
  detail?: TraceSessionDetail;
  loading: boolean;
  enabled: boolean;
}) {
  if (loading) {
    return (
      <div className="flex justify-center py-10">
        <RefreshCw className="size-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  if (!detail || detail.traces.length === 0) {
    return <EmptyState enabled={enabled} />;
  }

  const traces = [...detail.traces].sort(
    (a, b) =>
      (b.turnIndex ?? 0) - (a.turnIndex ?? 0) || b.createdAt - a.createdAt,
  );

  return (
    <div className="space-y-3">
      {traces.map((trace) => (
        <TraceTurnCard key={trace.traceId} trace={trace} />
      ))}
    </div>
  );
}

function UsageDetails({
  session,
  detail,
}: {
  session: TraceSessionSummary;
  detail?: TraceSessionDetail;
}) {
  const { t } = useTranslation();
  const traces = [...(detail?.traces ?? [])].sort(
    (a, b) =>
      (b.turnIndex ?? 0) - (a.turnIndex ?? 0) || b.createdAt - a.createdAt,
  );
  const totalTokens =
    session.totalInputTokens +
    session.totalOutputTokens +
    session.totalCacheReadTokens +
    session.totalCacheCreationTokens;

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-4">
        <Metric
          icon={Database}
          label={t("sessionTraces.tokens", { defaultValue: "Tokens" })}
          value={formatTokens(totalTokens)}
        />
        <Metric
          icon={Activity}
          label="Input"
          value={formatTokens(session.totalInputTokens)}
        />
        <Metric
          icon={MessageSquare}
          label="Output"
          value={formatTokens(session.totalOutputTokens)}
        />
        <Metric
          icon={Clock}
          label={t("sessionTraces.avgLatency", {
            defaultValue: "Avg latency",
          })}
          value={
            session.avgLatencyMs == null ? "--" : `${session.avgLatencyMs}ms`
          }
        />
      </div>

      <Card>
        <CardHeader className="py-3">
          <CardTitle className="text-sm">
            {t("sessionTraces.perTurnUsage", {
              defaultValue: "Per-turn Usage",
            })}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-2">
          {traces.length === 0 ? (
            <div className="text-sm text-muted-foreground">
              {t("sessionTraces.noTurnUsage", {
                defaultValue: "No per-turn trace usage has been captured yet.",
              })}
            </div>
          ) : (
            traces.map((trace) => (
              <div
                key={trace.traceId}
                className="grid gap-2 rounded-md border bg-muted/25 p-2 text-xs md:grid-cols-[80px_1fr_1fr_1fr_1fr]"
              >
                <div className="font-medium">#{trace.turnIndex ?? "-"}</div>
                <div>
                  <span className="text-muted-foreground">Input </span>
                  <span className="font-mono">
                    {formatTokens(trace.inputTokens)}
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">Output </span>
                  <span className="font-mono">
                    {formatTokens(trace.outputTokens)}
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">Cache </span>
                  <span className="font-mono">
                    {formatTokens(
                      trace.cacheReadTokens + trace.cacheCreationTokens,
                    )}
                  </span>
                </div>
                <div>
                  <span className="text-muted-foreground">Latency </span>
                  <span className="font-mono">
                    {trace.latencyMs == null ? "--" : `${trace.latencyMs}ms`}
                  </span>
                </div>
              </div>
            ))
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export function SessionTracesPage({
  target,
}: {
  target?: SessionTraceTarget | null;
}) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [search, setSearch] = useState("");
  const [appFilter, setAppFilter] = useState("all");
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(
    null,
  );
  useEffect(() => {
    if (!target?.sessionId) return;
    setSearch(target.sessionId);
    setSelectedSessionId(target.sessionId);
    if (target.appType) {
      setAppFilter(
        target.appType === "claude-desktop" ? "claude" : target.appType,
      );
    }
  }, [target]);

  const filters = useMemo(
    () => ({ search: search.trim() || undefined, limit: 200 }),
    [search],
  );
  const { data: settingsData } = useSessionTraceSettingsQuery();
  const settings = settingsData ?? DEFAULT_TRACE_SETTINGS;
  const updateSettings = useUpdateSessionTraceSettingsMutation();
  const {
    data: allSessions = [],
    isLoading,
    refetch,
  } = useTraceSessionsQuery(filters);
  const sessions = useMemo(
    () =>
      appFilter === "all"
        ? allSessions
        : allSessions.filter((session) => session.appType === appFilter),
    [allSessions, appFilter],
  );
  const appFilters = useMemo(() => {
    const counts = allSessions.reduce<Record<string, number>>(
      (acc, session) => {
        acc[session.appType] = (acc[session.appType] ?? 0) + 1;
        return acc;
      },
      { all: allSessions.length },
    );
    const appTypes = Object.keys(counts).sort((a, b) => {
      const orderA = PREFERRED_AGENT_ORDER.indexOf(a);
      const orderB = PREFERRED_AGENT_ORDER.indexOf(b);
      return (
        (orderA === -1 ? Number.MAX_SAFE_INTEGER : orderA) -
          (orderB === -1 ? Number.MAX_SAFE_INTEGER : orderB) ||
        a.localeCompare(b)
      );
    });
    return appTypes.map((appType) => ({
      appType,
      count: counts[appType] ?? 0,
    }));
  }, [allSessions]);

  useEffect(() => {
    if (sessions.length === 0) {
      setSelectedSessionId(null);
      return;
    }
    if (
      !selectedSessionId ||
      !sessions.some((s) => s.sessionId === selectedSessionId)
    ) {
      setSelectedSessionId(sessions[0].sessionId);
    }
  }, [selectedSessionId, sessions]);

  const selectedSession = useMemo(
    () => sessions.find((s) => s.sessionId === selectedSessionId) ?? null,
    [selectedSessionId, sessions],
  );
  const detailRequest = useMemo(
    () =>
      selectedSession
        ? {
            sessionId: selectedSession.sessionId,
            appType: selectedSession.appType,
          }
        : undefined,
    [selectedSession],
  );
  const {
    data: detail,
    isLoading: detailLoading,
    refetch: refetchDetail,
  } = useTraceSessionDetailQuery(detailRequest);
  const enabled = Boolean(settings.enabled);

  const saveSettings = async (next: SessionTraceSettings) => {
    try {
      await updateSettings.mutateAsync(next);
      toast.success(
        t("sessionTraces.settingsSaved", {
          defaultValue: "Session Traces 设置已保存",
        }),
      );
      await queryClient.invalidateQueries({ queryKey: ["traceSessions"] });
      await queryClient.invalidateQueries({
        queryKey: ["traceSessionDetail"],
      });
    } catch (error) {
      toast.error(String(error));
    }
  };

  const handleEnabledChange = (checked: boolean) => {
    const mode: SessionTraceMode = checked ? "summary" : "off";
    void saveSettings({
      ...settings,
      enabled: checked,
      mode,
      captureRequestJson: false,
      captureResponseJson: false,
    });
  };

  const handleModeChange = (mode: SessionTraceMode) => {
    void saveSettings({
      ...settings,
      enabled: mode !== "off",
      mode,
      captureRequestJson: mode === "full",
      captureResponseJson: mode === "full",
    });
  };

  const handleRefresh = async () => {
    await queryClient.invalidateQueries({ queryKey: ["traceSessions"] });
    await queryClient.invalidateQueries({ queryKey: ["traceSessionDetail"] });
    await refetch();
    if (detailRequest) {
      await refetchDetail();
    }
  };

  return (
    <div className="mx-auto flex h-full min-h-0 flex-col px-4 sm:px-6">
      <div className="mb-4 flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex min-w-0 flex-wrap items-center gap-2">
            <Activity className="size-5 text-primary" />
            <h1 className="text-lg font-semibold">
              {t("sessionTraces.title", { defaultValue: "Session Traces" })}
            </h1>
            <Badge variant={enabled ? "default" : "secondary"}>
              {enabled
                ? settings?.mode || "summary"
                : t("sessionTraces.off", { defaultValue: "Off" })}
            </Badge>
          </div>
          <p className="mt-1 max-w-2xl text-sm text-muted-foreground">
            {t("sessionTraces.subtitle", {
              defaultValue:
                "Analyze context pressure, tool usage, and per-turn model usage.",
            })}
          </p>
        </div>
        <div className="flex min-w-0 flex-wrap items-center justify-end gap-2">
          <div className="flex h-9 items-center gap-2 rounded-md border bg-card px-3">
            <span className="text-xs text-muted-foreground">
              {t("sessionTraces.enableShort", { defaultValue: "Enable" })}
            </span>
            <Switch
              checked={enabled}
              disabled={updateSettings.isPending}
              onCheckedChange={handleEnabledChange}
            />
          </div>
          <Select
            value={enabled ? settings.mode : "off"}
            disabled={updateSettings.isPending}
            onValueChange={(value) =>
              handleModeChange(value as SessionTraceMode)
            }
          >
            <SelectTrigger className="h-9 w-[132px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="off">
                {t("sessionTraces.modeOff", { defaultValue: "关闭" })}
              </SelectItem>
              <SelectItem value="summary">
                {t("sessionTraces.modeSummary", { defaultValue: "Summary" })}
              </SelectItem>
              <SelectItem value="full">
                {t("sessionTraces.modeFull", { defaultValue: "Full" })}
              </SelectItem>
            </SelectContent>
          </Select>
          <Button
            variant="outline"
            size="sm"
            className="h-9 gap-2 whitespace-nowrap"
            onClick={() => void handleRefresh()}
          >
            <RefreshCw
              className={cn(
                "size-3.5",
                (isLoading || detailLoading) && "animate-spin",
              )}
            />
            {t("common.refresh")}
          </Button>
        </div>
      </div>

      {!enabled && (
        <div className="mb-4 rounded-lg border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-800 dark:text-amber-200">
          {t("sessionTraces.offBanner", {
            defaultValue:
              "Session Traces 当前关闭。此页面会显示已有 usage 聚合，但不会记录新的上下文 trace。",
          })}
        </div>
      )}

      {target?.sessionId && (
        <div className="mb-4 rounded-lg border bg-muted/45 px-4 py-3 text-sm text-muted-foreground">
          {t("sessionTraces.analyzingLegacySession", {
            defaultValue:
              "正在查看旧会话 {{sessionId}} 的 traces 与 usage 统计。",
            sessionId: target.sessionId.slice(0, 8),
          })}
        </div>
      )}

      <div className="grid min-h-0 flex-1 gap-4 md:grid-cols-[minmax(260px,34%)_minmax(0,1fr)]">
        <Card className="flex min-h-0 flex-col overflow-hidden">
          <CardHeader className="border-b px-3 py-3">
            <div className="flex items-center justify-between gap-2">
              <CardTitle className="text-sm">
                {t("sessionTraces.traceSessions", {
                  defaultValue: "Trace Sessions",
                })}
              </CardTitle>
              <Badge variant="secondary">{sessions.length}</Badge>
            </div>
            <div className="relative mt-2">
              <Search className="absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder={t("sessionTraces.searchPlaceholder", {
                  defaultValue: "Search sessions, models...",
                })}
                className="h-8 pl-8 text-sm"
              />
            </div>
            <div className="mt-2 flex flex-wrap gap-1.5">
              {appFilters.map((filter) => (
                <button
                  key={filter.appType}
                  type="button"
                  onClick={() => setAppFilter(filter.appType)}
                  className={cn(
                    "inline-flex h-7 items-center gap-1.5 rounded-md border px-2 text-xs transition-colors",
                    appFilter === filter.appType
                      ? "border-primary/40 bg-primary/10 text-primary"
                      : "border-transparent bg-muted/50 text-muted-foreground hover:bg-muted",
                  )}
                >
                  <span>{getAppLabel(filter.appType, t)}</span>
                  <span className="font-mono text-[10px] opacity-70">
                    {filter.count}
                  </span>
                </button>
              ))}
            </div>
          </CardHeader>
          <CardContent className="min-h-0 flex-1 p-0">
            <ScrollArea className="h-full">
              <div className="space-y-1 p-2">
                {isLoading ? (
                  <div className="flex justify-center py-10">
                    <RefreshCw className="size-5 animate-spin text-muted-foreground" />
                  </div>
                ) : sessions.length === 0 ? (
                  <EmptyState enabled={enabled} />
                ) : (
                  sessions.map((session) => (
                    <TraceSessionItem
                      key={`${session.appType}:${session.sessionId}`}
                      session={session}
                      selected={session.sessionId === selectedSessionId}
                      onSelect={setSelectedSessionId}
                    />
                  ))
                )}
              </div>
            </ScrollArea>
          </CardContent>
        </Card>

        <Card className="flex min-h-0 flex-col overflow-hidden">
          {!selectedSession ? (
            <EmptyState enabled={enabled} />
          ) : (
            <>
              <CardHeader className="border-b px-4 py-3">
                <div className="flex flex-wrap items-start justify-between gap-3">
                  <div className="min-w-0">
                    <div className="flex min-w-0 items-center gap-2">
                      <ProviderIcon
                        icon={getProviderIconName(selectedSession.appType)}
                        name={selectedSession.appType}
                        size={20}
                      />
                      <CardTitle className="line-clamp-2 text-base leading-snug">
                        {getTraceSessionTitle(selectedSession, detail)}
                      </CardTitle>
                    </div>
                    <div className="mt-1 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-muted-foreground">
                      <span>{getAppLabel(selectedSession.appType, t)}</span>
                      {selectedSession.model && (
                        <span>{selectedSession.model}</span>
                      )}
                      <span className="font-mono">
                        {selectedSession.sessionId.slice(0, 8)}
                      </span>
                      <span>
                        {formatTimestamp(selectedSession.lastSeenAt * 1000)}
                      </span>
                    </div>
                  </div>
                  <Badge variant="outline">
                    {selectedSession.turnCount} turns
                  </Badge>
                </div>
              </CardHeader>
              <CardContent className="min-h-0 flex-1 overflow-y-auto p-4">
                <Tabs defaultValue="overview" className="space-y-4">
                  <TabsList className="flex h-auto flex-wrap justify-start">
                    <TabsTrigger className="min-w-0 flex-1" value="overview">
                      {t("sessionTraces.tabsOverview", {
                        defaultValue: "Overview",
                      })}
                    </TabsTrigger>
                    <TabsTrigger className="min-w-0 flex-1" value="context">
                      {t("sessionTraces.tabsContext", {
                        defaultValue: "Context",
                      })}
                    </TabsTrigger>
                    <TabsTrigger className="min-w-0 flex-1" value="traces">
                      {t("sessionTraces.tabsTraces", {
                        defaultValue: "Traces",
                      })}
                    </TabsTrigger>
                    <TabsTrigger className="min-w-0 flex-1" value="usage">
                      {t("sessionTraces.tabsUsage", {
                        defaultValue: "Usage",
                      })}
                    </TabsTrigger>
                  </TabsList>
                  <TabsContent value="overview">
                    <SessionOverview
                      session={selectedSession}
                      detail={detail}
                    />
                  </TabsContent>
                  <TabsContent value="context">
                    <ContextBreakdown
                      detail={detail}
                      loading={detailLoading}
                      enabled={enabled}
                    />
                  </TabsContent>
                  <TabsContent value="traces">
                    <TraceTurns
                      detail={detail}
                      loading={detailLoading}
                      enabled={enabled}
                    />
                  </TabsContent>
                  <TabsContent value="usage">
                    <UsageDetails session={selectedSession} detail={detail} />
                  </TabsContent>
                </Tabs>
              </CardContent>
            </>
          )}
        </Card>
      </div>
    </div>
  );
}
