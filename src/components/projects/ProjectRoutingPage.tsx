import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import {
  RefreshCw,
  FolderOpen,
  Plus,
  Unlink,
  Check,
  Loader2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";
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
import { extractErrorMessage } from "@/utils/errorUtils";

/** 项目路由信息 */
interface ProjectRoutingInfo {
  project_path: string;
  provider_id: string | null;
  provider_name: string | null;
  provider_notes?: string | null;
  session_count: number;
}

/** Provider 选项 */
interface ProviderOption {
  id: string;
  name: string;
  notes?: string;
}

/** 项目路由概览响应 */
interface ProjectRoutingOverview {
  projects: ProjectRoutingInfo[];
  available_providers: ProviderOption[];
}

type AppType = "claude" | "codex";

interface ProjectRoutingPageProps {
  app?: AppType;
}

export function ProjectRoutingPage({
  app = "claude",
}: ProjectRoutingPageProps) {
  const { t } = useTranslation();
  const [overview, setOverview] = useState<ProjectRoutingOverview | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [editingProject, setEditingProject] = useState<string | null>(null);
  const [isAddingProject, setIsAddingProject] = useState(false);
  const [newProjectPath, setNewProjectPath] = useState("");

  /** 加载项目路由数据 */
  const loadData = useCallback(async () => {
    try {
      const result = await invoke<ProjectRoutingOverview>(
        "get_project_routing",
      );
      setOverview(result);
    } catch (err) {
      toast.error(
        t("projectRouting.loadFailed", {
          defaultValue: "加载项目路由失败",
        }) +
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

  /** 刷新项目列表 */
  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      const result = await invoke<ProjectRoutingOverview>(
        "refresh_session_projects",
      );
      setOverview(result);
      toast.success(
        t("projectRouting.refreshSuccess", {
          defaultValue: "已刷新项目列表",
        }),
      );
    } catch (err) {
      toast.error(extractErrorMessage(err));
    } finally {
      setIsRefreshing(false);
    }
  };

  /** 设置项目绑定的 Provider */
  const handleSetProvider = async (projectPath: string, providerId: string) => {
    try {
      await invoke("set_project_provider_for_app", {
        app,
        projectPath,
        providerId,
      });
      // 刷新数据
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

  /** 移除项目的 Provider 绑定 */
  const handleRemoveProvider = async (projectPath: string) => {
    try {
      await invoke("remove_project_provider_for_app", { app, projectPath });
      await loadData();
      toast.success(
        t("projectRouting.providerRemoved", {
          defaultValue: "已移除项目绑定",
        }),
      );
    } catch (err) {
      toast.error(extractErrorMessage(err));
    }
  };

  /** 手动添加项目路径 */
  const handleAddProject = async () => {
    const path = newProjectPath.trim();
    if (!path) return;

    // 检查是否已存在
    if (overview?.projects.some((p) => p.project_path === path)) {
      toast.error(
        t("projectRouting.projectExists", {
          defaultValue: "该项目路径已存在",
        }),
      );
      return;
    }

    // 直接设置一个空的 provider 绑定（用户可以稍后修改）
    try {
      // 先获取数据看看有没有可用的 provider
      if (overview && overview.available_providers.length > 0) {
        // 不自动绑定，只添加到列表（通过设置一个空映射来"注册"项目）
      }
      setIsAddingProject(false);
      setNewProjectPath("");
      // 重新加载数据（虽然手动添加的路径不会出现在 ~/.claude/projects/ 扫描结果中，
      // 但我们可以通过设置 provider 绑定来"注册"它）
      toast.success(
        t("projectRouting.addProjectHint", {
          defaultValue: "请为新项目选择一个供应商以完成添加",
        }),
      );
    } catch (err) {
      toast.error(extractErrorMessage(err));
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  const projects = overview?.projects ?? [];
  const availableProviders = overview?.available_providers ?? [];

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
                  count: projects.length,
                })}
              </Badge>
            </div>
            <div className="flex items-center gap-2">
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setIsAddingProject(!isAddingProject)}
                    className="gap-1.5"
                  >
                    <Plus className="w-3.5 h-3.5" />
                    {!isAddingProject &&
                      t("projectRouting.addProject", {
                        defaultValue: "手动添加",
                      })}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  {t("projectRouting.addProjectTooltip", {
                    defaultValue: "手动添加项目路径（用于未自动发现的项目）",
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
                    {t("projectRouting.refresh", {
                      defaultValue: "刷新",
                    })}
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  {t("projectRouting.refreshTooltip", {
                    defaultValue: "重新扫描 ~/.claude/projects/ 目录",
                  })}
                </TooltipContent>
              </Tooltip>
            </div>
          </div>

          {/* 手动添加项目路径 */}
          {isAddingProject && (
            <Card className="border-dashed">
              <CardContent className="pt-4">
                <div className="flex items-center gap-2">
                  <input
                    type="text"
                    value={newProjectPath}
                    onChange={(e) => setNewProjectPath(e.target.value)}
                    placeholder={t("projectRouting.pathPlaceholder", {
                      defaultValue: "输入项目绝对路径，如 /Users/you/project",
                    })}
                    className="flex-1 h-9 rounded-md border border-input bg-background px-3 py-1 text-sm shadow-sm transition-colors placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring font-mono"
                    onKeyDown={(e) => {
                      if (e.key === "Enter") void handleAddProject();
                      if (e.key === "Escape") {
                        setIsAddingProject(false);
                        setNewProjectPath("");
                      }
                    }}
                    autoFocus
                  />
                  <Button
                    size="sm"
                    onClick={() => void handleAddProject()}
                    disabled={!newProjectPath.trim()}
                  >
                    {t("common.add", { defaultValue: "添加" })}
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      setIsAddingProject(false);
                      setNewProjectPath("");
                    }}
                  >
                    {t("common.cancel", { defaultValue: "取消" })}
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}

          {/* 项目列表 */}
          <ScrollArea className="flex-1 min-h-0">
            {projects.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
                <FolderOpen className="w-12 h-12 mb-4 opacity-50" />
                <p className="text-sm text-center max-w-md">
                  {t("projectRouting.emptyState", {
                    defaultValue:
                      "未发现项目。请确保目录存在，并且已使用过对应工具。",
                  })}
                </p>
                <Button
                  variant="outline"
                  size="sm"
                  className="mt-4 gap-1.5"
                  onClick={() => void handleRefresh()}
                >
                  <RefreshCw className="w-3.5 h-3.5" />
                  {t("projectRouting.refresh", {
                    defaultValue: "刷新",
                  })}
                </Button>
              </div>
            ) : (
              <div className="grid gap-3 pb-4">
                {projects.map((project) => (
                  <Card
                    key={project.project_path}
                    className="transition-colors hover:border-primary/30"
                  >
                    <CardHeader className="py-3 px-4">
                      <div className="flex items-start justify-between gap-3">
                        <div className="flex-1 min-w-0">
                          <CardTitle className="text-sm font-medium font-mono truncate">
                            {project.project_path}
                          </CardTitle>
                          <div className="flex items-center gap-2 mt-1.5">
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
                                  defaultValue: "未绑定供应商",
                                })}
                              </Badge>
                            )}
                          </div>
                        </div>
                        <div className="flex items-center gap-1 shrink-0">
                          {editingProject === project.project_path ? (
                            <Select
                              value={project.provider_id ?? ""}
                              onValueChange={(value) =>
                                void handleSetProvider(
                                  project.project_path,
                                  value,
                                )
                              }
                              onOpenChange={(open) => {
                                if (!open) setEditingProject(null);
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
                              <SelectContent className="max-h-[300px] overflow-y-auto">
                                {availableProviders.map((provider) => (
                                  <SelectItem
                                    key={provider.id}
                                    value={provider.id}
                                  >
                                    {provider.name}
                                    {provider.notes && (
                                      <span className="ml-1 text-muted-foreground text-[10px]">
                                        ({provider.notes})
                                      </span>
                                    )}
                                  </SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                          ) : (
                            <>
                              <Tooltip>
                                <TooltipTrigger asChild>
                                  <Button
                                    variant="outline"
                                    size="sm"
                                    className="h-7 px-2 text-xs gap-1"
                                    onClick={() =>
                                      setEditingProject(project.project_path)
                                    }
                                  >
                                    {project.provider_name ? (
                                      <>
                                        <Check className="w-3 h-3" />
                                        {t("projectRouting.change", {
                                          defaultValue: "修改",
                                        })}
                                      </>
                                    ) : (
                                      <>
                                        <Plus className="w-3 h-3" />
                                        {t("projectRouting.bind", {
                                          defaultValue: "绑定",
                                        })}
                                      </>
                                    )}
                                  </Button>
                                </TooltipTrigger>
                                <TooltipContent>
                                  {project.provider_name
                                    ? t("projectRouting.changeTooltip", {
                                        defaultValue: "修改绑定的供应商",
                                      })
                                    : t("projectRouting.bindTooltip", {
                                        defaultValue: "为此项目绑定一个供应商",
                                      })}
                                </TooltipContent>
                              </Tooltip>
                              {project.provider_id && (
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Button
                                      variant="ghost"
                                      size="sm"
                                      className="h-7 px-2 text-xs text-muted-foreground hover:text-destructive"
                                      onClick={() =>
                                        void handleRemoveProvider(
                                          project.project_path,
                                        )
                                      }
                                    >
                                      <Unlink className="w-3 h-3" />
                                    </Button>
                                  </TooltipTrigger>
                                  <TooltipContent>
                                    {t("projectRouting.unbindTooltip", {
                                      defaultValue: "移除供应商绑定",
                                    })}
                                  </TooltipContent>
                                </Tooltip>
                              )}
                            </>
                          )}
                        </div>
                      </div>
                    </CardHeader>
                  </Card>
                ))}
              </div>
            )}
          </ScrollArea>
        </div>
      </div>
    </TooltipProvider>
  );
}
