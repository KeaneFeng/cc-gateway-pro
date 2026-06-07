import { useCallback, useEffect, useMemo, useState } from "react";
import {
  Download,
  Copy,
  ExternalLink,
  Github,
  Globe,
  Info,
  Loader2,
  RefreshCw,
  Terminal,
  CheckCircle2,
  AlertCircle,
  ArrowUpCircle,
  ChevronDown,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { getVersion } from "@tauri-apps/api/app";
import { settingsApi } from "@/lib/api";
import { useUpdate } from "@/contexts/UpdateContext";
import { Badge } from "@/components/ui/badge";
import { motion } from "framer-motion";
import appIcon from "@/assets/icons/app-icon.png";
import { APP_ICON_MAP } from "@/config/appConfig";
import type { AppId } from "@/lib/api/types";
import { extractErrorMessage } from "@/utils/errorUtils";
import { isWindows } from "@/lib/platform";
import { isUpdateAvailable } from "@/lib/version";

interface AboutSectionProps {
  isPortable: boolean;
}

interface ToolVersion {
  name: string;
  version: string | null;
  latest_version: string | null;
  error: string | null;
  env_type: "windows" | "wsl" | "macos" | "linux" | "unknown";
  wsl_distro: string | null;
}

const TOOL_NAMES = [
  "claude",
  "codex",
  "gemini",
  "opencode",
  "openclaw",
  "hermes",
] as const;
type ToolName = (typeof TOOL_NAMES)[number];
type ToolLifecycleAction = "install" | "update";

type WslShellPreference = {
  wslShell?: string | null;
  wslShellFlag?: string | null;
};

const WSL_SHELL_OPTIONS = ["sh", "bash", "zsh", "fish", "dash"] as const;
// UI-friendly order: login shell first.
const WSL_SHELL_FLAG_OPTIONS = ["-lic", "-lc", "-c"] as const;

const ENV_BADGE_CONFIG: Record<
  string,
  { labelKey: string; className: string }
> = {
  wsl: {
    labelKey: "settings.envBadge.wsl",
    className:
      "bg-orange-500/10 text-orange-600 dark:text-orange-400 border-orange-500/20",
  },
  windows: {
    labelKey: "settings.envBadge.windows",
    className:
      "bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/20",
  },
  macos: {
    labelKey: "settings.envBadge.macos",
    className:
      "bg-gray-500/10 text-gray-600 dark:text-gray-400 border-gray-500/20",
  },
  linux: {
    labelKey: "settings.envBadge.linux",
    className:
      "bg-green-500/10 text-green-600 dark:text-green-400 border-green-500/20",
  },
};

const TOOL_DISPLAY_NAMES: Record<ToolName, string> = {
  claude: "Claude Code",
  codex: "Codex",
  gemini: "Gemini CLI",
  opencode: "OpenCode",
  openclaw: "OpenClaw",
  hermes: "Hermes",
};

const TOOL_APP_IDS: Record<ToolName, AppId> = {
  claude: "claude",
  codex: "codex",
  gemini: "gemini",
  opencode: "opencode",
  openclaw: "openclaw",
  hermes: "hermes",
};

const posixScriptInstallCommand = (url: string) =>
  `bash -c 'tmp=$(mktemp) && curl -fsSL ${url} -o $tmp && bash $tmp; status=$?; rm -f $tmp; exit $status'`;

const HERMES_WINDOWS_INSTALL_SCRIPT =
  "irm https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.ps1 | iex";

const powershellEncodedCommand = (script: string): string => {
  let binary = "";
  for (let i = 0; i < script.length; i += 1) {
    const code = script.charCodeAt(i);
    binary += String.fromCharCode(code & 0xff, code >> 8);
  }
  return btoa(binary);
};

const HERMES_WINDOWS_INSTALL_COMMAND = `powershell -NoProfile -ExecutionPolicy Bypass -EncodedCommand ${powershellEncodedCommand(
  HERMES_WINDOWS_INSTALL_SCRIPT,
)}`;

const POSIX_ONE_CLICK_INSTALL_COMMANDS = `# Claude Code (Native install - recommended)
${posixScriptInstallCommand("https://claude.ai/install.sh")} || npm i -g @anthropic-ai/claude-code@latest
# Codex
npm i -g @openai/codex@latest
# Gemini CLI
npm i -g @google/gemini-cli@latest
# OpenCode
${posixScriptInstallCommand("https://opencode.ai/install")} || npm i -g opencode-ai@latest
# OpenClaw
npm i -g openclaw@latest
# Hermes
${posixScriptInstallCommand("https://raw.githubusercontent.com/NousResearch/hermes-agent/main/scripts/install.sh")}`;

const WINDOWS_ONE_CLICK_INSTALL_COMMANDS = `# Claude Code
npm i -g @anthropic-ai/claude-code@latest
# Codex
npm i -g @openai/codex@latest
# Gemini CLI
npm i -g @google/gemini-cli@latest
# OpenCode
npm i -g opencode-ai@latest
# OpenClaw
npm i -g openclaw@latest
# Hermes
${HERMES_WINDOWS_INSTALL_COMMAND}`;

const ONE_CLICK_INSTALL_COMMANDS = isWindows()
  ? WINDOWS_ONE_CLICK_INSTALL_COMMANDS
  : POSIX_ONE_CLICK_INSTALL_COMMANDS;

export function AboutSection({ isPortable }: AboutSectionProps) {
  const { t } = useTranslation();
  const [version, setVersion] = useState<string | null>(null);
  const [isLoadingVersion, setIsLoadingVersion] = useState(true);
  const [isDownloading, setIsDownloading] = useState(false);
  const [toolVersions, setToolVersions] = useState<ToolVersion[]>([]);
  const [isLoadingTools, setIsLoadingTools] = useState(true);
  const [showInstallCommands, setShowInstallCommands] = useState(false);

  const {
    hasUpdate,
    updateInfo,
    updateHandle,
    checkUpdate,
    resetDismiss,
    isChecking,
  } = useUpdate();

  const [wslShellByTool, setWslShellByTool] = useState<
    Record<string, WslShellPreference>
  >({});
  const [loadingTools, setLoadingTools] = useState<Record<string, boolean>>({});
  const [busyTools, setBusyTools] = useState<Record<string, boolean>>({});

  const toolVersionByName = useMemo(() => {
    return new Map(toolVersions.map((tool) => [tool.name, tool]));
  }, [toolVersions]);

  const updatableToolNames = useMemo(
    () =>
      TOOL_NAMES.filter((toolName) => {
        const tool = toolVersionByName.get(toolName);
        return isUpdateAvailable(tool?.version, tool?.latest_version);
      }),
    [toolVersionByName],
  );

  const isAnyBusy = Object.values(busyTools).some(Boolean);

  const refreshToolVersions = useCallback(
    async (
      toolNames: ToolName[],
      wslOverrides?: Record<string, WslShellPreference>,
    ): Promise<ToolVersion[]> => {
      if (toolNames.length === 0) return [];

      setLoadingTools((prev) => {
        const next = { ...prev };
        for (const name of toolNames) next[name] = true;
        return next;
      });

      try {
        const updated = await settingsApi.getToolVersions(
          toolNames,
          wslOverrides,
        );

        setToolVersions((prev) => {
          if (prev.length === 0) return updated;
          const byName = new Map(updated.map((t) => [t.name, t]));
          const merged = prev.map((t) => byName.get(t.name) ?? t);
          const existing = new Set(prev.map((t) => t.name));
          for (const u of updated) {
            if (!existing.has(u.name)) merged.push(u);
          }
          return merged;
        });

        return updated;
      } catch (error) {
        console.error("[AboutSection] Failed to refresh tools", error);
        return [];
      } finally {
        setLoadingTools((prev) => {
          const next = { ...prev };
          for (const name of toolNames) next[name] = false;
          return next;
        });
      }
    },
    [],
  );

  const loadAllToolVersions = useCallback(async () => {
    setIsLoadingTools(true);
    try {
      const versions = await settingsApi.getToolVersions(
        [...TOOL_NAMES],
        wslShellByTool,
      );
      setToolVersions(versions);
    } catch (error) {
      console.error("[AboutSection] Failed to load tool versions", error);
    } finally {
      setIsLoadingTools(false);
    }
  }, [wslShellByTool]);

  const handleToolShellChange = async (toolName: ToolName, value: string) => {
    const wslShell = value === "auto" ? null : value;
    const nextPref: WslShellPreference = {
      ...(wslShellByTool[toolName] ?? {}),
      wslShell,
    };
    setWslShellByTool((prev) => ({ ...prev, [toolName]: nextPref }));
    await refreshToolVersions([toolName], { [toolName]: nextPref });
  };

  const handleToolShellFlagChange = async (
    toolName: ToolName,
    value: string,
  ) => {
    const wslShellFlag = value === "auto" ? null : value;
    const nextPref: WslShellPreference = {
      ...(wslShellByTool[toolName] ?? {}),
      wslShellFlag,
    };
    setWslShellByTool((prev) => ({ ...prev, [toolName]: nextPref }));
    await refreshToolVersions([toolName], { [toolName]: nextPref });
  };

  const handleToolAction = useCallback(
    async (toolName: ToolName, action: ToolLifecycleAction) => {
      setBusyTools((prev) => ({ ...prev, [toolName]: true }));
      try {
        const previousTool = toolVersionByName.get(toolName);
        const previousVersion = previousTool?.version ?? null;

        await settingsApi.runToolLifecycleAction(
          [toolName],
          action,
          wslShellByTool,
        );

        // Refresh version after action and check result
        const refreshed = await refreshToolVersions([toolName], wslShellByTool);
        const tool = refreshed.find((t) => t.name === toolName);

        if (
          action === "update" &&
          previousVersion &&
          tool?.version === previousVersion
        ) {
          // Version didn't change after update - might be a no-op or failure
          const latestVersion = tool.latest_version;
          if (isUpdateAvailable(tool.version, latestVersion)) {
            toast.warning(
              t("settings.toolActionPartial", {
                success: 0,
                failed: toolName,
              }),
              { closeButton: true },
            );
          } else {
            toast.success(
              t("settings.toolActionUpdateSuccess", { tool: toolName }),
              { closeButton: true },
            );
          }
        } else if (action === "install" && !tool?.version) {
          // Install command ran but version still not detected
          toast.warning(
            t("settings.toolActionPartial", {
              success: 0,
              failed: toolName,
            }),
            { closeButton: true },
          );
        } else {
          toast.success(
            t(
              action === "install"
                ? "settings.toolActionInstallSuccess"
                : "settings.toolActionUpdateSuccess",
              { tool: toolName },
            ),
            { closeButton: true },
          );
        }
      } catch (error) {
        console.error(
          `[AboutSection] Tool ${action} failed for ${toolName}`,
          error,
        );
        toast.error(
          t("settings.toolActionFailed", {
            tool: toolName,
            error: extractErrorMessage(error) || String(error),
          }),
          { closeButton: true },
        );
      } finally {
        setBusyTools((prev) => ({ ...prev, [toolName]: false }));
      }
    },
    [wslShellByTool, refreshToolVersions, toolVersionByName, t],
  );

  const handleUpdateAll = useCallback(async () => {
    if (updatableToolNames.length === 0) {
      toast.info(t("settings.allToolsUpToDate"), { closeButton: true });
      return;
    }

    setBusyTools((prev) => {
      const next = { ...prev };
      for (const name of updatableToolNames) next[name] = true;
      return next;
    });

    const results: { tool: string; success: boolean; error?: string }[] = [];

    for (const toolName of updatableToolNames) {
      try {
        await settingsApi.runToolLifecycleAction(
          [toolName],
          "update",
          wslShellByTool,
        );
        results.push({ tool: toolName, success: true });
      } catch (error) {
        results.push({
          tool: toolName,
          success: false,
          error: extractErrorMessage(error) || String(error),
        });
      }
    }

    await refreshToolVersions(updatableToolNames, wslShellByTool);

    setBusyTools((prev) => {
      const next = { ...prev };
      for (const name of updatableToolNames) next[name] = false;
      return next;
    });

    const failed = results.filter((r) => !r.success);
    if (failed.length === 0) {
      toast.success(
        t("settings.toolActionUpdateAllSuccess", { count: results.length }),
        { closeButton: true },
      );
    } else if (failed.length === results.length) {
      toast.error(t("settings.toolActionUpdateAllFailed"), {
        closeButton: true,
      });
    } else {
      toast.warning(
        t("settings.toolActionPartial", {
          success: results.length - failed.length,
          failed: failed.map((r) => r.tool).join(", "),
        }),
        { closeButton: true },
      );
    }
  }, [updatableToolNames, wslShellByTool, refreshToolVersions, t]);

  useEffect(() => {
    let active = true;
    const load = async () => {
      try {
        const [appVersion] = await Promise.all([
          getVersion(),
          ...(isWindows() ? [] : [loadAllToolVersions()]),
        ]);

        if (active) {
          setVersion(appVersion);
        }
      } catch (error) {
        console.error("[AboutSection] Failed to load info", error);
        if (active) {
          setVersion(null);
        }
      } finally {
        if (active) {
          setIsLoadingVersion(false);
        }
      }
    };

    void load();
    return () => {
      active = false;
    };
    // Mount-only: loadAllToolVersions is intentionally excluded to avoid
    // re-fetching all tools whenever wslShellByTool changes. Single-tool
    // refreshes are handled by refreshToolVersions in the shell/flag handlers.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const handleOpenReleaseNotes = useCallback(async () => {
    try {
      const targetVersion = updateInfo?.availableVersion ?? version ?? "";
      const displayVersion = targetVersion.startsWith("v")
        ? targetVersion
        : targetVersion
          ? `v${targetVersion}`
          : "";

      if (!displayVersion) {
        await settingsApi.openExternal(
          "https://github.com/KeaneFeng/cc-gateway-pro/releases",
        );
        return;
      }

      await settingsApi.openExternal(
        `https://github.com/KeaneFeng/cc-gateway-pro/releases/tag/${displayVersion}`,
      );
    } catch (error) {
      console.error("[AboutSection] Failed to open release notes", error);
      toast.error(t("settings.openReleaseNotesFailed"));
    }
  }, [t, updateInfo?.availableVersion, version]);

  const handleCheckUpdate = useCallback(async () => {
    if (hasUpdate && updateHandle) {
      if (isPortable) {
        try {
          await settingsApi.checkUpdates();
        } catch (error) {
          console.error("[AboutSection] Portable update failed", error);
        }
        return;
      }

      setIsDownloading(true);
      try {
        resetDismiss();
        await updateHandle.downloadAndInstall();
        await settingsApi.restart();
      } catch (error) {
        console.error("[AboutSection] Update failed", error);
        toast.error(t("settings.updateFailed"));
        try {
          await settingsApi.checkUpdates();
        } catch (fallbackError) {
          console.error(
            "[AboutSection] Failed to open fallback updater",
            fallbackError,
          );
        }
      } finally {
        setIsDownloading(false);
      }
      return;
    }

    try {
      const available = await checkUpdate();
      if (!available) {
        toast.success(t("settings.upToDate"), { closeButton: true });
      }
    } catch (error) {
      console.error("[AboutSection] Check update failed", error);
      toast.error(t("settings.checkUpdateFailed"));
    }
  }, [checkUpdate, hasUpdate, isPortable, resetDismiss, t, updateHandle]);

  const handleCopyInstallCommands = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(ONE_CLICK_INSTALL_COMMANDS);
      toast.success(t("settings.installCommandsCopied"), { closeButton: true });
    } catch (error) {
      console.error("[AboutSection] Failed to copy install commands", error);
      toast.error(t("settings.installCommandsCopyFailed"));
    }
  }, [t]);

  const displayVersion = version ?? t("common.unknown");

  return (
    <motion.section
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      className="space-y-6"
    >
      <header className="space-y-1">
        <h3 className="text-sm font-medium">{t("common.about")}</h3>
        <p className="text-xs text-muted-foreground">
          {t("settings.aboutHint")}
        </p>
      </header>

      <motion.div
        initial={{ opacity: 0, scale: 0.98 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ duration: 0.3, delay: 0.1 }}
        className="rounded-xl border border-border bg-gradient-to-br from-card/80 to-card/40 p-6 space-y-5 shadow-sm"
      >
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div className="space-y-2">
            <div className="flex items-center gap-2">
              <img src={appIcon} alt={t("app.title")} className="h-5 w-5" />
              <h4 className="text-lg font-semibold text-foreground">
                {t("app.title")}
              </h4>
            </div>
            <div className="flex items-center gap-2">
              <Badge variant="outline" className="gap-1.5 bg-background/80">
                <span className="text-muted-foreground">
                  {t("common.version")}
                </span>
                {isLoadingVersion ? (
                  <Loader2 className="h-3 w-3 animate-spin" />
                ) : (
                  <span className="font-medium">{`v${displayVersion}`}</span>
                )}
              </Badge>
              {isPortable && (
                <Badge variant="secondary" className="gap-1.5">
                  <Info className="h-3 w-3" />
                  {t("settings.portableMode")}
                </Badge>
              )}
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() =>
                settingsApi.openExternal(
                  "https://github.com/KeaneFeng/cc-gateway-pro",
                )
              }
              className="h-8 gap-1.5 text-xs"
            >
              <Globe className="h-3.5 w-3.5" />
              {t("settings.officialWebsite")}
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() =>
                settingsApi.openExternal(
                  "https://github.com/KeaneFeng/cc-gateway-pro",
                )
              }
              className="h-8 gap-1.5 text-xs"
            >
              <Github className="h-3.5 w-3.5" />
              {t("settings.github")}
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleOpenReleaseNotes}
              className="h-8 gap-1.5 text-xs"
            >
              <ExternalLink className="h-3.5 w-3.5" />
              {t("settings.releaseNotes")}
            </Button>
            <Button
              type="button"
              size="sm"
              onClick={handleCheckUpdate}
              disabled={isChecking || isDownloading}
              className="h-8 gap-1.5 text-xs"
            >
              {isDownloading ? (
                <>
                  <Loader2 className="h-3.5 w-3.5 animate-spin" />
                  {t("settings.updating")}
                </>
              ) : hasUpdate ? (
                <>
                  <Download className="h-3.5 w-3.5" />
                  {t("settings.updateTo", {
                    version: updateInfo?.availableVersion ?? "",
                  })}
                </>
              ) : isChecking ? (
                <>
                  <RefreshCw className="h-3.5 w-3.5 animate-spin" />
                  {t("settings.checking")}
                </>
              ) : (
                <>
                  <RefreshCw className="h-3.5 w-3.5" />
                  {t("settings.checkForUpdates")}
                </>
              )}
            </Button>
          </div>
        </div>

        {hasUpdate && updateInfo && (
          <motion.div
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: "auto" }}
            className="rounded-lg bg-primary/10 border border-primary/20 px-4 py-3 text-sm"
          >
            <p className="font-medium text-primary mb-1">
              {t("settings.updateAvailable", {
                version: updateInfo.availableVersion,
              })}
            </p>
            {updateInfo.notes && (
              <p className="text-muted-foreground line-clamp-3 leading-relaxed">
                {updateInfo.notes}
              </p>
            )}
          </motion.div>
        )}
      </motion.div>

      {!isWindows() && (
        <div className="space-y-3">
          <div className="flex flex-col gap-2 px-1 sm:flex-row sm:items-center sm:justify-between">
            <h3 className="text-sm font-medium">
              {t("settings.localEnvCheck")}
            </h3>
            <div className="flex flex-wrap items-center gap-2">
              <Button
                size="sm"
                variant="outline"
                className="h-7 gap-1.5 text-xs"
                onClick={() => handleUpdateAll()}
                disabled={
                  isLoadingTools || isAnyBusy || updatableToolNames.length === 0
                }
                aria-label={t("settings.updateAll")}
              >
                {isAnyBusy ? (
                  <Loader2
                    className="h-3.5 w-3.5 animate-spin"
                    aria-hidden="true"
                  />
                ) : (
                  <ArrowUpCircle className="h-3.5 w-3.5" aria-hidden="true" />
                )}
                {t("settings.updateAllTools", {
                  count: updatableToolNames.length,
                })}
              </Button>
              <Button
                size="sm"
                variant="outline"
                className="h-7 gap-1.5 text-xs"
                onClick={() => loadAllToolVersions()}
                disabled={isLoadingTools || isAnyBusy}
              >
                <RefreshCw
                  className={
                    isLoadingTools ? "h-3.5 w-3.5 animate-spin" : "h-3.5 w-3.5"
                  }
                />
                {isLoadingTools ? t("common.refreshing") : t("common.refresh")}
              </Button>
            </div>
          </div>

          <div className="grid gap-3 px-1 sm:grid-cols-2 xl:grid-cols-3">
            {TOOL_NAMES.map((toolName, index) => {
              const tool = toolVersionByName.get(toolName);
              const appConfig = APP_ICON_MAP[TOOL_APP_IDS[toolName]];
              const displayName = TOOL_DISPLAY_NAMES[toolName];
              const isToolVersionLoading =
                isLoadingTools || Boolean(loadingTools[toolName]);
              const isOutdated = isUpdateAvailable(
                tool?.version,
                tool?.latest_version,
              );
              const action: ToolLifecycleAction | null =
                isToolVersionLoading || !tool?.version
                  ? !tool?.version && !isToolVersionLoading
                    ? "install"
                    : null
                  : isOutdated
                    ? "update"
                    : null;
              const isToolBusy = busyTools[toolName];
              const title = tool?.version || tool?.error || t("common.unknown");

              return (
                <motion.div
                  key={toolName}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.3, delay: 0.15 + index * 0.04 }}
                  className="flex min-h-[140px] flex-col gap-3 rounded-xl border border-border bg-gradient-to-br from-card/80 to-card/40 p-4 shadow-sm transition-colors hover:border-primary/30"
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="flex min-w-0 items-center gap-2">
                      <span className="flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-background/80 text-muted-foreground">
                        {appConfig?.icon ?? <Terminal className="h-4 w-4" />}
                      </span>
                      <div className="min-w-0">
                        <div className="truncate text-sm font-medium">
                          {displayName}
                        </div>
                        {tool?.env_type && ENV_BADGE_CONFIG[tool.env_type] && (
                          <span
                            className={`mt-1 inline-flex w-fit text-[9px] px-1.5 py-0.5 rounded-full border ${ENV_BADGE_CONFIG[tool.env_type].className}`}
                          >
                            {t(ENV_BADGE_CONFIG[tool.env_type].labelKey)}
                            {tool.wsl_distro ? ` · ${tool.wsl_distro}` : ""}
                          </span>
                        )}
                      </div>
                    </div>
                    {isToolVersionLoading ? (
                      <Loader2 className="mt-1 h-4 w-4 animate-spin text-muted-foreground" />
                    ) : tool?.version ? (
                      isOutdated ? (
                        <span className="mt-1 shrink-0 rounded-full border border-yellow-500/20 bg-yellow-500/10 px-1.5 py-0.5 text-[10px] text-yellow-600 dark:text-yellow-400">
                          {t("settings.updateAvailableShort")}
                        </span>
                      ) : (
                        <CheckCircle2 className="mt-1 h-4 w-4 shrink-0 text-green-500" />
                      )
                    ) : (
                      <AlertCircle className="mt-1 h-4 w-4 shrink-0 text-yellow-500" />
                    )}
                  </div>

                  <div className="space-y-1.5 text-xs">
                    <div className="flex items-center justify-between gap-3">
                      <span className="text-muted-foreground">
                        {t("settings.currentVersion")}
                      </span>
                      <span
                        className="min-w-0 truncate font-mono text-foreground"
                        title={title}
                      >
                        {isToolVersionLoading
                          ? t("common.loading")
                          : tool?.version
                            ? tool.version
                            : t("common.notInstalled")}
                      </span>
                    </div>
                    <div className="flex items-center justify-between gap-3">
                      <span className="text-muted-foreground">
                        {t("settings.latestVersion")}
                      </span>
                      <span className="min-w-0 truncate font-mono text-foreground">
                        {isToolVersionLoading
                          ? t("common.loading")
                          : tool?.latest_version || t("common.unknown")}
                      </span>
                    </div>
                    {!isToolVersionLoading && !tool?.version && tool?.error && (
                      <div className="truncate text-[11px] text-muted-foreground">
                        {tool.error}
                      </div>
                    )}
                  </div>

                  {tool?.env_type === "wsl" && (
                    <div className="flex flex-wrap gap-2">
                      <Select
                        value={wslShellByTool[toolName]?.wslShell || "auto"}
                        onValueChange={(v) =>
                          handleToolShellChange(toolName, v)
                        }
                        disabled={
                          isLoadingTools || loadingTools[toolName] || isAnyBusy
                        }
                      >
                        <SelectTrigger className="h-7 w-[82px] text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="auto">
                            {t("common.auto")}
                          </SelectItem>
                          {WSL_SHELL_OPTIONS.map((shell) => (
                            <SelectItem key={shell} value={shell}>
                              {shell}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                      <Select
                        value={wslShellByTool[toolName]?.wslShellFlag || "auto"}
                        onValueChange={(v) =>
                          handleToolShellFlagChange(toolName, v)
                        }
                        disabled={
                          isLoadingTools || loadingTools[toolName] || isAnyBusy
                        }
                      >
                        <SelectTrigger className="h-7 w-[82px] text-xs">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="auto">
                            {t("common.auto")}
                          </SelectItem>
                          {WSL_SHELL_FLAG_OPTIONS.map((flag) => (
                            <SelectItem key={flag} value={flag}>
                              {flag}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>
                  )}

                  <div className="mt-auto flex items-center justify-end">
                    {isToolVersionLoading ? (
                      <span className="text-xs text-muted-foreground">
                        {t("common.loading")}
                      </span>
                    ) : action ? (
                      <Button
                        size="sm"
                        variant={action === "install" ? "outline" : "default"}
                        className="h-7 gap-1.5 text-xs"
                        onClick={() => handleToolAction(toolName, action)}
                        disabled={isToolVersionLoading || isAnyBusy}
                      >
                        {isToolBusy ? (
                          <Loader2 className="h-3.5 w-3.5 animate-spin" />
                        ) : action === "install" ? (
                          <Download className="h-3.5 w-3.5" />
                        ) : (
                          <ArrowUpCircle className="h-3.5 w-3.5" />
                        )}
                        {action === "install"
                          ? t("settings.toolInstall")
                          : t("settings.toolUpdate")}
                      </Button>
                    ) : (
                      <span className="text-xs text-muted-foreground">
                        {t("settings.toolReady")}
                      </span>
                    )}
                  </div>
                </motion.div>
              );
            })}
          </div>
        </div>
      )}

      {!isWindows() && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, delay: 0.3 }}
          className="space-y-3"
        >
          <button
            type="button"
            onClick={() => setShowInstallCommands((v) => !v)}
            aria-expanded={showInstallCommands}
            className="flex w-full items-center gap-1.5 px-1 text-sm font-medium text-foreground transition-colors hover:text-primary"
          >
            <ChevronDown
              className={`h-3.5 w-3.5 transition-transform ${
                showInstallCommands ? "" : "-rotate-90"
              }`}
            />
            {t("settings.manualInstallCommands")}
          </button>
          {showInstallCommands && (
            <div className="rounded-xl border border-border bg-gradient-to-br from-card/80 to-card/40 p-4 space-y-3 shadow-sm">
              <div className="flex items-center justify-between gap-2">
                <p className="text-xs text-muted-foreground">
                  {t("settings.oneClickInstallHint")}
                </p>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={handleCopyInstallCommands}
                  className="h-7 gap-1.5 text-xs"
                >
                  <Copy className="h-3.5 w-3.5" />
                  {t("common.copy")}
                </Button>
              </div>
              <pre className="text-xs font-mono bg-background/80 px-3 py-2.5 rounded-lg border border-border/60 overflow-x-auto">
                {ONE_CLICK_INSTALL_COMMANDS}
              </pre>
            </div>
          )}
        </motion.div>
      )}
    </motion.section>
  );
}
