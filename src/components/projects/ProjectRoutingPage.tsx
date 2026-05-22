import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { RefreshCw, FolderOpen, Plus, Unlink, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger } from "@/components/ui/tooltip";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { extractErrorMessage } from "@/utils/errorUtils";

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

type AppType = "claude" | "codex";

export function ProjectRoutingPage() {
  const { t } = useTranslation();
  const [app, setApp] = useState<AppType>("claude");
  const [overview, setOverview] = useState<ProjectRoutingOverview | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [editingProject, setEditingProject] = useState<string | null>(null);

  const loadData = useCallback(async () => {
    try {
      const result = await invoke<ProjectRoutingOverview>("get_project_routing_for_app", { app });
      setOverview(result);
    } catch (err) {
      toast.error(t("projectRouting.loadFailed", { defaultValue: "Load failed" }) + ": " + extractErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, [app, t]);

  useEffect(() => { void loadData(); }, [loadData]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      const result = await invoke<ProjectRoutingOverview>("get_project_routing_for_app", { app });
      setOverview(result);
      toast.success(t("projectRouting.refreshSuccess", { defaultValue: "Refreshed" }));
    } catch (err) {
      toast.error(extractErrorMessage(err));
    } finally {
      setIsRefreshing(false);
    }
  };

  const handleSetProvider = async (projectPath: string, providerId: string) => {
    try {
      await invoke("set_project_provider_for_app", { app, projectPath, providerId });
      await loadData();
      setEditingProject(null);
      toast.success(t("projectRouting.providerUpdated", { defaultValue: "Updated" }));
    } catch (err) {
      toast.error(extractErrorMessage(err));
    }
  };

  const handleRemoveProvider = async (projectPath: string) => {
    try {
      await invoke("remove_project_provider_for_app", { app, projectPath });
      await loadData();
      toast.success(t("projectRouting.providerRemoved", { defaultValue: "Removed" }));
    } catch (err) {
      toast.error(extractErrorMessage(err));
    }
  };

  if (isLoading) {
    return <div className="flex items-center justify-center h-full"><Loader2 className="w-6 h-6 animate-spin text-muted-foreground" /></div>;
  }

  const projects = overview?.projects ?? [];
  const availableProviders = overview?.available_providers ?? [];

  return (
    <TooltipProvider>
      <div className="mx-auto px-4 sm:px-6 flex flex-col h-full min-h-0">
        <div className="flex-1 overflow-hidden flex flex-col gap-4">
          {/* App Tabs */}
          <Tabs value={app} onValueChange={(v) => setApp(v as AppType)} className="w-full">
            <div className="flex items-center justify-between pt-2">
              <TabsList>
                <TabsTrigger value="claude">{t("projectRouting.tabClaude", { defaultValue: "Claude Code" })}</TabsTrigger>
                <TabsTrigger value="codex">{t("projectRouting.tabCodex", { defaultValue: "Codex" })}</TabsTrigger>
              </TabsList>
              <div className="flex items-center gap-2">
                <Badge variant="secondary" className="text-xs">
                  {t("projectRouting.projectCount", { defaultValue: "{{count}} projects", count: projects.length })}
                </Badge>
                <Tooltip>
                  <TooltipTrigger asChild>
                    <Button variant="outline" size="sm" onClick={() => void handleRefresh()} disabled={isRefreshing} className="gap-1.5">
                      <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? "animate-spin" : ""}`} />
                      {t("projectRouting.refresh", { defaultValue: "Refresh" })}
                    </Button>
                  </TooltipTrigger>
                  <TooltipContent>
                    {t("projectRouting.refreshTooltip", { defaultValue: app === "claude" ? "Scan ~/.claude/projects/" : "Scan ~/.codex/sessions/" })}
                  </TooltipContent>
                </Tooltip>
              </div>
            </div>

            <TabsContent value="claude" className="mt-0">
              <ScrollArea className="flex-1 min-h-0">
                {projects.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
                    <FolderOpen className="w-12 h-12 mb-4 opacity-50" />
                    <p className="text-sm text-center max-w-md">
                      {t("projectRouting.emptyStateClaude", { defaultValue: "No Claude projects found. Ensure ~/.claude/projects/ exists and Claude Code has been used." })}
                    </p>
                    <Button variant="outline" size="sm" className="mt-4 gap-1.5" onClick={() => void handleRefresh()}>
                      <RefreshCw className="w-3.5 h-3.5" />
                      {t("projectRouting.refresh", { defaultValue: "Refresh" })}
                    </Button>
                  </div>
                ) : (
                  <div className="grid gap-3 pb-4">
                    {projects.map((project) => (
                      <Card key={project.project_path} className="transition-colors hover:border-primary/30">
                        <CardHeader className="py-3 px-4">
                          <div className="flex items-start justify-between gap-3">
                            <div className="flex-1 min-w-0">
                              <CardTitle className="text-sm font-medium font-mono truncate">{project.project_path}</CardTitle>
                              <div className="flex items-center gap-2 mt-1.5">
                                {project.session_count > 0 && (
                                  <Badge variant="secondary" className="text-[10px] px-1.5 py-0">
                                    {t("projectRouting.sessionCount", { defaultValue: "{{count}} sessions", count: project.session_count })}
                                  </Badge>
                                )}
                                {project.provider_name ? (
                                  <Badge variant="default" className="text-[10px] px-1.5 py-0 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20">
                                    {project.provider_name}
                                    {project.provider_notes && <span className="ml-1 opacity-70">({project.provider_notes})</span>}
                                  </Badge>
                                ) : (
                                  <Badge variant="outline" className="text-[10px] px-1.5 py-0 text-muted-foreground">
                                    {t("projectRouting.noProvider", { defaultValue: "Default provider" })}
                                  </Badge>
                                )}
                              </div>
                            </div>
                            <div className="flex items-center gap-1.5 shrink-0">
                              {editingProject === project.project_path ? (
                                <Select
                                  defaultValue={project.provider_id || undefined}
                                  onValueChange={(value) => void handleSetProvider(project.project_path, value)}
                                  onOpenChange={(open) => { if (!open) setEditingProject(null); }}
                                  defaultOpen
                                >
                                  <SelectTrigger className="w-[180px] h-8 text-xs">
                                    <SelectValue placeholder={t("projectRouting.selectProvider", { defaultValue: "Select provider" })} />
                                  </SelectTrigger>
                                  <SelectContent>
                                    {availableProviders.map((provider) => (
                                      <SelectItem key={provider.id} value={provider.id} className="text-xs">
                                        {provider.name}
                                        {provider.notes && <span className="ml-1 opacity-70">({provider.notes})</span>}
                                      </SelectItem>
                                    ))}
                                  </SelectContent>
                                </Select>
                              ) : (
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Button variant="ghost" size="sm" onClick={() => setEditingProject(project.project_path)} className="h-8 w-8 p-0">
                                      <Plus className="w-3.5 h-3.5" />
                                    </Button>
                                  </TooltipTrigger>
                                  <TooltipContent>{t("projectRouting.changeProvider", { defaultValue: "Change provider" })}</TooltipContent>
                                </Tooltip>
                              )}
                              {project.provider_id && (
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Button variant="ghost" size="sm" onClick={() => void handleRemoveProvider(project.project_path)} className="h-8 w-8 p-0 text-destructive hover:text-destructive">
                                      <Unlink className="w-3.5 h-3.5" />
                                    </Button>
                                  </TooltipTrigger>
                                  <TooltipContent>{t("projectRouting.removeProvider", { defaultValue: "Remove binding" })}</TooltipContent>
                                </Tooltip>
                              )}
                            </div>
                          </div>
                        </CardHeader>
                      </Card>
                    ))}
                  </div>
                )}
              </ScrollArea>
            </TabsContent>

            <TabsContent value="codex" className="mt-0">
              <ScrollArea className="flex-1 min-h-0">
                {projects.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-20 text-muted-foreground">
                    <FolderOpen className="w-12 h-12 mb-4 opacity-50" />
                    <p className="text-sm text-center max-w-md">
                      {t("projectRouting.emptyStateCodex", { defaultValue: "No Codex sessions found. Ensure ~/.codex/sessions/ exists and Codex CLI has been used." })}
                    </p>
                    <Button variant="outline" size="sm" className="mt-4 gap-1.5" onClick={() => void handleRefresh()}>
                      <RefreshCw className="w-3.5 h-3.5" />
                      {t("projectRouting.refresh", { defaultValue: "Refresh" })}
                    </Button>
                  </div>
                ) : (
                  <div className="grid gap-3 pb-4">
                    {projects.map((project) => (
                      <Card key={project.project_path} className="transition-colors hover:border-primary/30">
                        <CardHeader className="py-3 px-4">
                          <div className="flex items-start justify-between gap-3">
                            <div className="flex-1 min-w-0">
                              <CardTitle className="text-sm font-medium font-mono truncate">{project.project_path}</CardTitle>
                              <div className="flex items-center gap-2 mt-1.5">
                                {project.session_count > 0 && (
                                  <Badge variant="secondary" className="text-[10px] px-1.5 py-0">
                                    {t("projectRouting.sessionCount", { defaultValue: "{{count}} sessions", count: project.session_count })}
                                  </Badge>
                                )}
                                {project.provider_name ? (
                                  <Badge variant="default" className="text-[10px] px-1.5 py-0 bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20">
                                    {project.provider_name}
                                    {project.provider_notes && <span className="ml-1 opacity-70">({project.provider_notes})</span>}
                                  </Badge>
                                ) : (
                                  <Badge variant="outline" className="text-[10px] px-1.5 py-0 text-muted-foreground">
                                    {t("projectRouting.noProvider", { defaultValue: "Default provider" })}
                                  </Badge>
                                )}
                              </div>
                            </div>
                            <div className="flex items-center gap-1.5 shrink-0">
                              {editingProject === project.project_path ? (
                                <Select
                                  defaultValue={project.provider_id || undefined}
                                  onValueChange={(value) => void handleSetProvider(project.project_path, value)}
                                  onOpenChange={(open) => { if (!open) setEditingProject(null); }}
                                  defaultOpen
                                >
                                  <SelectTrigger className="w-[180px] h-8 text-xs">
                                    <SelectValue placeholder={t("projectRouting.selectProvider", { defaultValue: "Select provider" })} />
                                  </SelectTrigger>
                                  <SelectContent>
                                    {availableProviders.map((provider) => (
                                      <SelectItem key={provider.id} value={provider.id} className="text-xs">
                                        {provider.name}
                                        {provider.notes && <span className="ml-1 opacity-70">({provider.notes})</span>}
                                      </SelectItem>
                                    ))}
                                  </SelectContent>
                                </Select>
                              ) : (
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Button variant="ghost" size="sm" onClick={() => setEditingProject(project.project_path)} className="h-8 w-8 p-0">
                                      <Plus className="w-3.5 h-3.5" />
                                    </Button>
                                  </TooltipTrigger>
                                  <TooltipContent>{t("projectRouting.changeProvider", { defaultValue: "Change provider" })}</TooltipContent>
                                </Tooltip>
                              )}
                              {project.provider_id && (
                                <Tooltip>
                                  <TooltipTrigger asChild>
                                    <Button variant="ghost" size="sm" onClick={() => void handleRemoveProvider(project.project_path)} className="h-8 w-8 p-0 text-destructive hover:text-destructive">
                                      <Unlink className="w-3.5 h-3.5" />
                                    </Button>
                                  </TooltipTrigger>
                                  <TooltipContent>{t("projectRouting.removeProvider", { defaultValue: "Remove binding" })}</TooltipContent>
                                </Tooltip>
                              )}
                            </div>
                          </div>
                        </CardHeader>
                      </Card>
                    ))}
                  </div>
                )}
              </ScrollArea>
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </TooltipProvider>
  );
}
