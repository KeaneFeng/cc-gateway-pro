import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Loader2, Copy, Trash2, MessageSquare } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { ConfirmDialog } from "@/components/ConfirmDialog";
import { extractErrorMessage } from "@/utils/errorUtils";
import { projectRoutingApi, type AppType } from "@/lib/api/project-routing";
import type { SessionMeta } from "@/types";

interface ProjectSessionListProps {
  app: AppType;
  projectPath: string;
}

export function ProjectSessionList({
  app,
  projectPath,
}: ProjectSessionListProps) {
  const { t } = useTranslation();
  const [sessions, setSessions] = useState<SessionMeta[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [deleteTarget, setDeleteTarget] = useState<SessionMeta | null>(null);

  const loadSessions = useCallback(async () => {
    try {
      const result = await projectRoutingApi.getSessionsForProject(
        app,
        projectPath,
      );
      setSessions(result);
    } catch (err) {
      toast.error(extractErrorMessage(err));
    } finally {
      setIsLoading(false);
    }
  }, [app, projectPath]);

  useEffect(() => {
    void loadSessions();
  }, [loadSessions]);

  // 生成恢复会话的命令
  const getResumeCommand = (session: SessionMeta): string => {
    if (app === "claude") {
      return `claude --resume ${session.sessionId}`;
    } else {
      return `codex resume ${session.sessionId}`;
    }
  };

  // 复制命令到剪贴板
  const handleCopyCommand = async (session: SessionMeta) => {
    const command = getResumeCommand(session);
    try {
      await navigator.clipboard.writeText(command);
      toast.success(
        t("projectRouting.commandCopied", {
          defaultValue: "命令已复制到剪贴板",
        }),
      );
    } catch (err) {
      toast.error(extractErrorMessage(err));
    }
  };

  // 删除会话
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
      // 重新加载会话列表
      await loadSessions();
    } catch (err) {
      toast.error(extractErrorMessage(err));
    } finally {
      setDeleteTarget(null);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-8">
        <Loader2 className="w-6 h-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  // 如果没有会话，显示空状态
  if (sessions.length === 0) {
    return (
      <div className="py-4 text-center text-sm text-muted-foreground">
        {t("projectRouting.noSessions", {
          defaultValue: "该项目暂无会话",
        })}
      </div>
    );
  }

  return (
    <TooltipProvider>
      <div className="space-y-2">
        {sessions.map((session) => (
          <div
            key={session.sessionId}
            className="flex items-center gap-2 p-2 rounded-md hover:bg-muted/50"
          >
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <MessageSquare className="w-4 h-4 text-muted-foreground shrink-0" />
                <p className="text-sm font-medium truncate">
                  {session.title || session.sessionId}
                </p>
              </div>
              {session.projectDir && (
                <p className="text-xs text-muted-foreground truncate ml-6">
                  {session.projectDir}
                </p>
              )}
            </div>
            <div className="flex items-center gap-1 shrink-0">
              {/* 复制恢复命令 */}
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 w-7 p-0"
                    onClick={() => void handleCopyCommand(session)}
                  >
                    <Copy className="w-3.5 h-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p className="font-mono text-xs">
                    {getResumeCommand(session)}
                  </p>
                </TooltipContent>
              </Tooltip>

              {/* 删除会话 */}
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    className="h-7 w-7 p-0 text-destructive hover:text-destructive"
                    onClick={() => setDeleteTarget(session)}
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  {t("projectRouting.deleteSession", {
                    defaultValue: "删除会话",
                  })}
                </TooltipContent>
              </Tooltip>
            </div>
          </div>
        ))}
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
