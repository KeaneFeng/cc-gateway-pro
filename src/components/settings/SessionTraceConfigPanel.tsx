import { useMemo } from "react";
import { toast } from "sonner";
import { ShieldCheck, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  usePruneSessionTracesMutation,
  useSessionTraceSettingsQuery,
  useUpdateSessionTraceSettingsMutation,
} from "@/lib/query/session-traces";
import type {
  SessionTraceMode,
  SessionTraceSettings,
} from "@/types/session-traces";

const DEFAULT_SETTINGS: SessionTraceSettings = {
  enabled: false,
  mode: "off",
  retentionDays: 7,
  maxResponseTextChars: 2000,
  captureRequestJson: false,
  captureResponseJson: false,
  redactSensitiveValues: true,
};

const RETENTION_DAY_OPTIONS = [3, 7, 14, 30, 90] as const;

export function SessionTraceConfigPanel() {
  const { t } = useTranslation();
  const { data, isLoading } = useSessionTraceSettingsQuery();
  const updateMutation = useUpdateSessionTraceSettingsMutation();
  const pruneMutation = usePruneSessionTracesMutation();
  const settings = data ?? DEFAULT_SETTINGS;

  const mode = useMemo<SessionTraceMode>(() => {
    if (!settings.enabled) return "off";
    return settings.mode;
  }, [settings.enabled, settings.mode]);

  const save = async (next: SessionTraceSettings) => {
    try {
      await updateMutation.mutateAsync(next);
      toast.success(
        t("sessionTraces.settingsSaved", {
          defaultValue: "Session Traces 设置已保存",
        }),
      );
    } catch (error) {
      toast.error(String(error));
    }
  };

  const handleEnabledChange = (enabled: boolean) => {
    const nextMode: SessionTraceMode = enabled ? "summary" : "off";
    void save({
      ...settings,
      enabled,
      mode: nextMode,
      captureRequestJson: false,
      captureResponseJson: false,
    });
  };

  const handleModeChange = (nextMode: SessionTraceMode) => {
    void save({
      ...settings,
      enabled: nextMode !== "off",
      mode: nextMode,
      captureRequestJson: nextMode === "full",
      captureResponseJson: nextMode === "full",
    });
  };

  const handleRetentionChange = (value: string) => {
    const retentionDays = Number.parseInt(value, 10);
    if (!Number.isFinite(retentionDays)) return;
    void save({
      ...settings,
      retentionDays,
    });
  };

  const handlePrune = async () => {
    try {
      const result = await pruneMutation.mutateAsync();
      toast.success(
        t("sessionTraces.prunedOldTraces", {
          defaultValue:
            "已清理 {{deleted}} 条旧 Trace，并压缩 {{compacted}} 条超大历史记录。",
          deleted: result.deleted,
          compacted: result.compacted,
        }),
      );
    } catch (error) {
      toast.error(String(error));
    }
  };

  if (isLoading) return null;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between gap-4">
        <div className="space-y-0.5">
          <Label>
            {t("sessionTraces.enable", {
              defaultValue: "启用 Session Traces",
            })}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t("sessionTraces.enableDescription", {
              defaultValue:
                "开启后才会记录新的会话上下文摘要、工具调用和每轮 usage。",
            })}
          </p>
        </div>
        <Switch
          checked={settings.enabled}
          disabled={updateMutation.isPending}
          onCheckedChange={handleEnabledChange}
        />
      </div>

      <div className="flex items-center justify-between gap-4">
        <div className="space-y-0.5">
          <Label>{t("sessionTraces.mode", { defaultValue: "采集模式" })}</Label>
          <p className="text-xs text-muted-foreground">
            {t("sessionTraces.modeDescription", {
              defaultValue:
                "Summary 只保存摘要；Full 会保存脱敏后的 request/response JSON。",
            })}
          </p>
        </div>
        <Select
          value={mode}
          disabled={updateMutation.isPending}
          onValueChange={(value) => handleModeChange(value as SessionTraceMode)}
        >
          <SelectTrigger className="w-[150px]">
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
      </div>

      <div className="flex items-center justify-between gap-4">
        <div className="space-y-0.5">
          <Label>
            {t("sessionTraces.retentionPeriod", {
              defaultValue: "数据保留周期",
            })}
          </Label>
          <p className="text-xs text-muted-foreground">
            {t("sessionTraces.retentionPeriodDescription", {
              defaultValue: "后台维护会自动清理超过该周期的 Session Traces。",
            })}
          </p>
        </div>
        <Select
          value={String(settings.retentionDays)}
          disabled={updateMutation.isPending}
          onValueChange={handleRetentionChange}
        >
          <SelectTrigger className="w-[150px]">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {RETENTION_DAY_OPTIONS.map((days) => (
              <SelectItem key={days} value={String(days)}>
                {t("sessionTraces.retentionDaysOption", {
                  defaultValue: "{{days}} 天",
                  days,
                })}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div className="grid gap-3 rounded-lg border bg-muted/40 p-4 text-xs text-muted-foreground">
        <div className="flex items-start gap-2">
          <ShieldCheck className="mt-0.5 size-4 shrink-0 text-emerald-500" />
          <p>
            {t("sessionTraces.privacyHint", {
              defaultValue:
                "Session Traces 默认关闭，数据只保存在本机。敏感 key 会被脱敏，但开启前仍应确认这是可信设备。",
            })}
          </p>
        </div>
        <div className="flex flex-wrap items-center gap-2">
          <span>
            {t("sessionTraces.retention", {
              defaultValue: "保留 {{days}} 天",
              days: settings.retentionDays,
            })}
          </span>
          <span>·</span>
          <span>
            {t("sessionTraces.previewLimit", {
              defaultValue: "响应预览 {{chars}} 字符",
              chars: settings.maxResponseTextChars,
            })}
          </span>
        </div>
      </div>

      <div className="flex justify-end">
        <Button
          variant="outline"
          size="sm"
          disabled={pruneMutation.isPending}
          className="gap-2"
          onClick={() => void handlePrune()}
        >
          <Trash2 className="size-3.5" />
          {t("sessionTraces.pruneOldTraces", {
            defaultValue: "清理旧 Traces",
          })}
        </Button>
      </div>
    </div>
  );
}
