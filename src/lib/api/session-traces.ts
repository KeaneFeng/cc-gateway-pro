import { invoke } from "@tauri-apps/api/core";
import type {
  SessionTraceSettings,
  TraceSessionDetail,
  TraceSessionDetailRequest,
  TraceSessionFilters,
  TraceSessionSummary,
} from "@/types/session-traces";

export const sessionTracesApi = {
  async getSettings(): Promise<SessionTraceSettings> {
    return await invoke("get_session_trace_settings");
  },

  async setSettings(
    settings: SessionTraceSettings,
  ): Promise<SessionTraceSettings> {
    return await invoke("set_session_trace_settings", { settings });
  },

  async listSessions(
    filters?: TraceSessionFilters,
  ): Promise<TraceSessionSummary[]> {
    return await invoke("list_trace_sessions", { filters });
  },

  async getSessionDetail(
    request: TraceSessionDetailRequest,
  ): Promise<TraceSessionDetail> {
    return await invoke("get_trace_session_detail", { request });
  },
};
