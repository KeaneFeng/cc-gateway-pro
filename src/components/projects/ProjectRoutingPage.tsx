import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import {
  RefreshCw,
  Unlink,
  Loader2,
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
import { extractErrorMessage } from "@/utils/errorUtils";
import { projectRoutingApi, type AppType } from "@/lib/api/project-routing";
import { ProjectSessionList } from "./ProjectSessionList";

interface ProjectRoutingInfo {
  project_path: string;
  provider_id: string | null;
  provider_name: string | null;
  provider_notes?: string | null;
  session_count: number;
}

interface ProviderOption {
  id: string;
  name: string;
  notes?: string;
}

interface ProjectRoutingOverview {
  projects: ProjectRoutingInfo[];
  available_providers: ProviderOption[];
}

interface ProjectRoutingPageProps {
  app: AppType;
}

export function ProjectRoutingPage({
  app = "claude",
}: ProjectRoutingPageProps) {
  const { t } = useTranslation();
  const [overview, setOverview] = useState<ProjectRoutingOverview | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [editingProject, setEditingProject] = useState<string | null>(null);
  const [expandedProject, setExpandedProject] = useState<string | null>(null);

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
                    defaultValue: "重新扫描项目目录",
                  })}
                </TooltipContent>
              </Tooltip>
            </div>
          </div>

          {/* 项目列表 */}
          <ScrollArea className="flex-1 min-h-0">
            {projects.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
                <p className="text-sm text-center max-w-md">
                  {t("projectRouting.emptyState", {
                    defaultValue:
                      "未发现项目。请确保目录存在，并且已使用过对应工具。",
                  })}
                </p>
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
                          <div className="flex items-center gap-2">
                            <Button
                              variant="ghost"
                              size="sm"
                              className="h-6 w-6 p-0"
                              onClick={() =>
                                setExpandedProject(
                                  expandedProject === project.project_path
                                    ? null
                                    : project.project_path,
                                )
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
                          <ProjectSessionList
                            app={app}
                            projectPath={project.project_path}
                          />
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
    </TooltipProvider>
  );
}
