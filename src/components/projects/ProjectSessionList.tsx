import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Loader2 } from "lucide-react";
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

  // 嵌入 SessionManagerPage 组件
  // 注意：SessionManagerPage 会显示所有会话，我们需要过滤
  // 由于 SessionManagerPage 不支持过滤 prop，我们需要创建一个包装
  // 这里先显示会话列表，后续可以优化为嵌入完整组件
  return (
    <div className="space-y-2">
      {sessions.map((session) => (
        <div
          key={session.sessionId}
          className="flex items-center gap-2 p-2 rounded-md hover:bg-muted/50 cursor-pointer"
        >
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium truncate">
              {session.title || session.sessionId}
            </p>
            {session.projectDir && (
              <p className="text-xs text-muted-foreground truncate">
                {session.projectDir}
              </p>
            )}
          </div>
        </div>
      ))}
    </div>
  );
}
