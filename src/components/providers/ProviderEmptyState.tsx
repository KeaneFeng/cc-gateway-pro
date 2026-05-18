import { Download, Users, RefreshCw, Loader2 } from "lucide-react";
import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import type { AppId } from "@/lib/api/types";

interface ProviderEmptyStateProps {
  appId: AppId;
  onCreate?: () => void;
  onImport?: () => void;
}

export function ProviderEmptyState({
  appId,
  onCreate,
  onImport,
}: ProviderEmptyStateProps) {
  const { t } = useTranslation();
  const [isSyncing, setIsSyncing] = useState(false);
  const showSnippetHint =
    appId === "claude" || appId === "codex" || appId === "gemini";

  const handleSyncFromCcSwitch = useCallback(async () => {
    setIsSyncing(true);
    try {
      const result: any = await invoke("sync_from_cc_switch");
      if (result.success) {
        toast.success(
          t("settings.syncFromCcSwitchSuccess", {
            defaultValue: `同步完成: ${result.syncedCount} 个供应商已导入`,
          }),
        );
        // 刷新页面数据
        window.location.reload();
      } else {
        toast.error(
          result.message ||
            t("settings.syncFromCcSwitchFailed", {
              defaultValue: "同步失败",
            }),
        );
      }
    } catch (err) {
      console.error("[SyncFromCcSwitch]", err);
      toast.error(
        t("settings.syncFromCcSwitchFailed", {
          defaultValue: "同步失败，请确认 cc-switch 已安装",
        }),
      );
    } finally {
      setIsSyncing(false);
    }
  }, [t]);

  return (
    <div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-border p-10 text-center">
      <div className="mb-4 flex h-16 w-16 items-center justify-center rounded-full bg-muted">
        <Users className="h-7 w-7 text-muted-foreground" />
      </div>
      <h3 className="text-lg font-semibold">{t("provider.noProviders")}</h3>
      <p className="mt-2 max-w-lg text-sm text-muted-foreground">
        {t("provider.noProvidersDescription")}
      </p>
      {showSnippetHint && (
        <p className="mt-1 max-w-lg text-sm text-muted-foreground">
          {t("provider.noProvidersDescriptionSnippet")}
        </p>
      )}
      <div className="mt-6 flex flex-col gap-2">
        {/* 从 cc-switch 同步供应商 */}
        <Button
          onClick={handleSyncFromCcSwitch}
          disabled={isSyncing}
          variant="default"
        >
          {isSyncing ? (
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
          ) : (
            <RefreshCw className="mr-2 h-4 w-4" />
          )}
          {isSyncing
            ? t("settings.syncingFromCcSwitch", {
                defaultValue: "正在从 cc-switch 同步...",
              })
            : t("settings.syncFromCcSwitch", {
                defaultValue: "从 cc-switch 同步供应商",
              })}
        </Button>
        {onImport && (
          <Button onClick={onImport} variant="outline">
            <Download className="mr-2 h-4 w-4" />
            {appId === "claude-desktop"
              ? t("provider.importFromClaude", {
                  defaultValue: "将 Claude Code 中已有的供应商导入",
                })
              : t("provider.importCurrent")}
          </Button>
        )}
        {onCreate && (
          <Button variant="outline" onClick={onCreate}>
            {t("provider.addProvider")}
          </Button>
        )}
      </div>
    </div>
  );
}
