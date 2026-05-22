import { invoke } from "@tauri-apps/api/core";
import type { SessionMeta } from "@/types";

export interface ProjectRoutingInfo {
  project_path: string;
  provider_id: string | null;
  provider_name: string | null;
  provider_notes?: string | null;
  session_count: number;
}

export interface ProviderOption {
  id: string;
  name: string;
  notes?: string;
}

export interface ProjectRoutingOverview {
  projects: ProjectRoutingInfo[];
  available_providers: ProviderOption[];
}

export type AppType = "claude" | "codex";

export const projectRoutingApi = {
  async getProjectRouting(app: AppType = "claude"): Promise<ProjectRoutingOverview> {
    return await invoke("get_project_routing_for_app", { app });
  },

  async setProjectProvider(
    app: AppType,
    projectPath: string,
    providerId: string,
  ): Promise<void> {
    await invoke("set_project_provider_for_app", {
      app,
      projectPath,
      providerId,
    });
  },

  async removeProjectProvider(
    app: AppType,
    projectPath: string,
  ): Promise<void> {
    await invoke("remove_project_provider_for_app", { app, projectPath });
  },

  async getSessionsForProject(
    app: AppType,
    projectPath: string,
  ): Promise<SessionMeta[]> {
    return await invoke("list_sessions_for_project", { app, projectPath });
  },

  async deleteSession(
    providerId: string,
    sessionId: string,
    sourcePath: string,
  ): Promise<boolean> {
    return await invoke("delete_session", {
      providerId,
      sessionId,
      sourcePath,
    });
  },
};
