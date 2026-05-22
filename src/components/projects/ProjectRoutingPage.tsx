import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  RefreshCw,
  FolderOpen,
  Plus,
  Unlink,
  Loader2,
  MessageSquare,
  Trash2,
  ChevronRight,
  ChevronDown,
  Pencil,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { extractErrorMessage } from "@/utils/errorUtils";
import { projectRoutingApi, type AppType } from "@/lib/api/project-routing";
import type { SessionMeta } from "@/types";
import { open } from "@tauri-apps/plugin-dialog";

interface ProjectRoutingPageProps {
  app: AppType;
}

export function ProjectRoutingPage({
  app = "claude",
}: ProjectRoutingPageProps) {
  const { t } = useTranslation();
  const [overview, setOverview] = useState<Awaited<
    ReturnType<typeof projectRoutingApi.getProjectRouting>
  > | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [editingProject, setEditingProject] = useState<string | null>(null);
  // 临时添加的项目（用户选择但还未绑定供应商）
  const [pendingProjects, setPendingProjects] = useState<Set<string>>(
    new Set(),
  );

  // Sessions 相关状态
  const [expandedProject, setExpandedProject] = useState<string | null>(null);
  const [projectSessions, setProjectSessions] = useState<
    Record<string, SessionMeta[]>
  >({});
  const [loadingSessions, setLoadingSessions] = useState<Set<string>>(
    new Set(),
  );
  const [deleteTarget, setDeleteTarget] = useState<SessionMeta | null>(null);

  const appLabel = app === "claude" ? "Claude Code" : "Codex";

  const loadData = useCallback(async () => {
    try {
      const result = await projectRoutingApi.getProjectRouting(app);
      setOverview(result);
    } catch (err) {
      toast.error(
        t("projectRouting.loadFailed", { defaultValue: "加载项目路由失败" }) +
          ": " +
          extractErrorMessage(err),
      );
    } finally {
      setIsLoading(false);
    }
  }, [app, t]);

  useEffect(() => {
    void loadData();
  }, [loadData]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      const result = await projectRoutingApi.getProjectRouting(app);
      setOverview(result);
      toast.success(
        t("projectRouting.refreshSuccess", { defaultValue: "已刷新项目列表" }),
      );
    } catch (err) {
      toast.error(extractErrorMessage(err));
    } finally {
      setIsRefreshing(false);
    }
  };

  const handleSetProvider = async (projectPath: string, providerId: string) => {
    try {
      await projectRoutingApi.setProjectProvider(app, projectPath, providerId);
      // 从临时列表移除
      setPendingProjects((prev) => {
        const next = new Set(prev);
        next.delete(projectPath);
        return next;
      });
      await loadData();
      setEditingProject(null);
      toast.success(
        t("projectRouting.providerUpdated", {
          defaultValue: "已更新项目绑定的供应商",
        }),
      );
    } catch (err) {
      toast.error(extractErrorMessage(err));
    }
  };

  const handleRemoveProvider = async (projectPath: string) => {
    try {
      await projectRoutingApi.removeProjectProvider(app, projectPath);
      await loadData();
      toast.success(
        t("projectRouting.providerRemoved", { defaultValue: "已移除项目绑定" }),
      );
    } catch (err) {
      toast.error(extractErrorMessage(err));
    }
  };

  // 打开 Finder 选择项目路径
  const handleSelectFolder = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: t("projectRouting.selectFolder", {
          defaultValue: `选择${appLabel}项目目录`,
        }),
      });
      if (selected) {
        const path = selected as string;
        // 检查是否已在已配置列表中
        const existingProjects = overview?.projects ?? [];
        const isAlreadyConfigured = existingProjects.some(
          (p) => p.project_path === path,
        );
        if (!isAlreadyConfigured) {
          // 添加到临时列表
          setPendingProjects((prev) => new Set(prev).add(path));
        }
        toast.success(
          t("projectRouting.folderSelected", {
            defaultValue: "已选择项目路径，请选择供应商以完成添加",
          }),
        );
        // 设置编辑状态，让用户选择 provider
        setEditingProject(path);
      }
    } catch (err) {
      toast.error(extractErrorMessage(err));
    }
  };

  // 加载项目的 sessions
  const handleToggleSessions = async (projectPath: string) => {
    if (expandedProject === projectPath) {
      setExpandedProject(null);
      return;
    }

    setExpandedProject(projectPath);

    if (projectSessions[projectPath]) {
      return;
    }

    setLoadingSessions((prev) => new Set(prev).add(projectPath));
    try {
      const sessions = await projectRoutingApi.getSessionsForProject(
        app,
        projectPath,
      );
      setProjectSessions((prev) => ({ ...prev, [projectPath]: sessions }));
    } catch (err) {
      toast.error(extractErrorMessage(err));
    } finally {
      setLoadingSessions((prev) => {
        const next = new Set(prev);
        next.delete(projectPath);
        return next;
      });
    }
  };

  // 删除 session
  const handleDeleteSession = async () => {
    if (!deleteTarget) return;

    try {
      await projectRoutingApi.deleteSession(
        deleteTarget.providerId,
        deleteTarget.sessionId,
        deleteTarget.sourcePath || "",
      );
      toast.success(
        t("projectRouting.sessionDeleted", { defaultValue: "会话已删除" }),
      );
      if (expandedProject) {
        const sessions = await projectRoutingApi.getSessionsForProject(
          app,
          expandedProject,
        );
        setProjectSessions((prev) => ({
          ...prev,
          [expandedProject]: sessions,
        }));
      }
    } catch (err) {
      toast.error(extractErrorMessage(err));
    } finally {
      setDeleteTarget(null);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  const configuredProjects = overview?.projects ?? [];
  const availableProviders = overview?.available_providers ?? [];

  // 合并已配置项目和临时项目
  const allProjects = [
    ...configuredProjects,
    ...Array.from(pendingProjects)
      .filter(
        (path) => !configuredProjects.some((p) => p.project_path === path),
      )
      .map((path) => ({
        project_path: path,
        provider_id: null,
        provider_name: null,
        provider_notes: null,
        session_count: 0,
      })),
  ];

  return (
    <TooltipProvider>
      <div className="mx-auto px-4 sm:px-6 flex flex-col h-full min-h-0">
        <div className="flex-1 overflow-hidden flex flex-col gap-4">
          {/* 顶部工具栏 */}
          <div className="flex items-center justify-between pt-2">
            <div className="flex items-center gap-2">
              <Badge variant="secondary" className="text-xs">
                {t("projectRouting.projectCount", {
                  defaultValue: "{{count}} 个项目",
                  count: allProjects.length,
                })}
              </Badge>
            </div>
            <div className="flex items-center gap-2">
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleSelectFolder}
                    className="gap-1.5"
                  >
                    <Plus className="w-3.5 h-3.5" />
                    {t("projectRouting.addProject", {
                      defaultValue: "添加项目",
                    })}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  {t("projectRouting.addProjectTooltip", {
                    defaultValue: "打开 Finder 选择项目目录",
                  })}
                </TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => void handleRefresh()}
                    disabled={isRefreshing}
                    className="gap-1.5"
                  >
                    <RefreshCw
                      className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin" : ""}`}
                    />
                    {t("projectRouting.refresh", { defaultValue: "刷新" })}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  {t("projectRouting.refreshTooltip", {
                    defaultValue:
                      app === "claude"
                        ? "重新扫描 ~/.claude/projects/ 目录"
                        : "重新扫描 ~/.codex/sessions/ 目录",
                  })}
                </TooltipContent>
              </Tooltip>
            </div>
          </div>

          {/* 项目列表 */}
          <ScrollArea className="flex-1 min-h-0">
            {allProjects.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
                <FolderOpen className="w-12 h-12 mb-4 opacity-50" />
                <p className="text-sm text-center max-w-md">
                  {t("projectRouting.emptyState", {
                    defaultValue:
                      app === "claude"
                        ? "未发现 Claude Code 项目。请确保 ~/.claude/projects/ 目录存在，并且已使用过 Claude Code。"
                        : "未发现 Codex 会话。请确保 ~/.codex/sessions/ 目录存在，并且已使用过 Codex CLI。",
                  })}
                </p>
                <Button
                  variant="outline"
                  size="sm"
                  className="mt-4 gap-1.5"
                  onClick={handleSelectFolder}
                >
                  <Plus className="w-3.5 h-3.5" />
                  {t("projectRouting.addProject", {
                    defaultValue: "添加项目",
                  })}
                </Button>
              </div>
            ) : (
              <div className="grid gap-3 pb-4">
                {allProjects.map((project) => (
                  <Card
                    key={project.project_path}
                    className="transition-colors hover:border-primary/30"
                  >
                    <CardHeader className="py-3 px-4">
                      <div className="flex items-start justify-between gap-3">
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-6 w-6 p-0"
                              onClick={() =>
                                void handleToggleSessions(project.project_path)
                              }
                            >
                              {expandedProject === project.project_path ? (
                                <ChevronDown className="w-4 h-4" />
                              ) : (
                                <ChevronRight className="w-4 h-4" />
                              )}
                            </Button>
                            <CardTitle className="text-sm font-medium font-mono truncate">
                              {project.project_path}
                            </CardTitle>
                          </div>
                          <div className="flex items-center gap-2 mt-1.5 ml-8">
                            {project.session_count > 0 && (
                              <Badge
                                variant="secondary"
                                className="text-[10px] px-1.5 py-0"
                              >
                                {t("projectRouting.sessionCount", {
                                  defaultValue: "{{count}} 个会话",
                                  count: project.session_count,
                                })}
                              </Badge>
                            )}
                            {project.provider_name ? (
                              <Badge
                                variant="default"
                                className="text-[10px] px-1.5 py-0 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20"
                              >
                                {project.provider_name}
                                {project.provider_notes && (
                                  <span className="ml-1 opacity-70">
                                    ({project.provider_notes})
                                  </span>
                                )}
                              </Badge>
                            ) : (
                              <Badge
                                variant="outline"
                                className="text-[10px] px-1.5 py-0 text-muted-foreground"
                              >
                                {t("projectRouting.noProvider", {
                                  defaultValue: "使用默认供应商",
                                })}
                              </Badge>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center gap-1.5 shrink-0">
                          {editingProject === project.project_path ? (
                            <Select
                              defaultValue={project.provider_id || undefined}
                              onValueChange={(value) =>
                                void handleSetProvider(
                                  project.project_path,
                                  value,
                                )
                              }
                              onOpenChange={(isOpen) => {
                                if (!isOpen) setEditingProject(null);
                              }}
                              defaultOpen
                            >
                              <SelectTrigger className="w-[180px] h-8 text-xs">
                                <SelectValue
                                  placeholder={t(
                                    "projectRouting.selectProvider",
                                    { defaultValue: "选择供应商" },
                                  )}
                                />
                              </SelectTrigger>
                              <SelectContent>
                                {availableProviders.map((provider) => (
                                  <SelectItem
                                    key={provider.id}
                                    value={provider.id}
                                    className="text-xs"
                                  >
                                    {provider.name}
                                    {provider.notes && (
                                      <span className="ml-1 opacity-70">
                                        ({provider.notes})
                                      </span>
                                    )}
                                  </SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                          ) : (
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() =>
                                    setEditingProject(project.project_path)
                                  }
                                  className="h-8 w-8 p-0"
                                >
                                  <Pencil className="w-3.5 h-3.5" />
                                </Button>
                              </TooltipTrigger>
                              <TooltipContent>
                                {t("projectRouting.changeProvider", {
                                  defaultValue: "更改供应商",
                                })}
                              </TooltipContent>
                            </Tooltip>
                          )}
                          {project.provider_id && (
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() =>
                                    void handleRemoveProvider(
                                      project.project_path,
                                    )
                                  }
                                  className="h-8 w-8 p-0 text-destructive hover:text-destructive"
                                >
                                  <Unlink className="w-3.5 h-3.5" />
                                </Button>
                              </TooltipTrigger>
                              <TooltipContent>
                                {t("projectRouting.removeProvider", {
                                  defaultValue: "移除绑定",
                                })}
                              </TooltipContent>
                            </Tooltip>
                          )}
                        </div>
                      </div>

                      {/* Sessions 列表 */}
                      {expandedProject === project.project_path && (
                        <div className="mt-3 ml-8 border-l-2 border-muted pl-4">
                          {loadingSessions.has(project.project_path) ? (
                            <div className="flex items-center gap-2 py-2 text-muted-foreground">
                              <Loader2 className="w-4 h-4 animate-spin" />
                              <span className="text-xs">
                                {t("projectRouting.loadingSessions", {
                                  defaultValue: "加载会话中...",
                                })}
                              </span>
                            </div>
                          ) : (
                            <div className="space-y-2">
                              {(projectSessions[project.project_path] ?? [])
                                .length === 0 ? (
                                <p className="text-xs text-muted-foreground py-2">
                                  {t("projectRouting.noSessions", {
                                    defaultValue: "该项目暂无会话",
                                  })}
                                </p>
                              ) : (
                                (
                                  projectSessions[project.project_path] ?? []
                                ).map((session) => (
                                  <div
                                    key={session.sessionId}
                                    className="flex items-center justify-between gap-2 p-2 rounded-md hover:bg-muted/50"
                                  >
                                    <div className="flex items-center gap-2 min-w-0 flex-1">
                                      <MessageSquare className="w-4 h-4 text-muted-foreground shrink-0" />
                                      <div className="min-w-0 flex-1">
                                        <p className="text-xs font-medium truncate">
                                          {session.title || session.sessionId}
                                        </p>
                                        {session.projectDir && (
                                          <p className="text-[10px] text-muted-foreground truncate">
                                            {session.projectDir}
                                          </p>
                                        )}
                                      </div>
                                    </div>
                                    <div className="flex items-center gap-1 shrink-0">
                                      <Button
                                        variant="ghost"
                                        size="sm"
                                        className="h-6 w-6 p-0 text-destructive hover:text-destructive"
                                        onClick={() => setDeleteTarget(session)}
                                      >
                                        <Trash2 className="w-3 h-3" />
                                      </Button>
                                    </div>
                                  </div>
                                ))
                              )}
                            </div>
                          )}
                        </div>
                      )}
                    </CardHeader>
                  </Card>
                ))}
              </div>
            )}
          </ScrollArea>
        </div>
      </div>

      {/* 删除确认对话框 */}
      <ConfirmDialog
        isOpen={!!deleteTarget}
        title={t("projectRouting.deleteSessionTitle", {
          defaultValue: "删除会话",
        })}
        message={t("projectRouting.deleteSessionDescription", {
          defaultValue: "此操作将永久删除该会话文件，无法恢复。是否继续？",
        })}
        confirmText={t("common.delete", { defaultValue: "删除" })}
        variant="destructive"
        onConfirm={() => void handleDeleteSession()}
        onCancel={() => setDeleteTarget(null)}
      />
    </TooltipProvider>
  );
}
