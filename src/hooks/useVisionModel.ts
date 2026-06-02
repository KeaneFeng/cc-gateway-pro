import { useState, useEffect, useCallback } from "react";

/**
 * CC-Gateway-Pro: Vision Model 状态管理 Hook
 *
 * 从 ProviderForm.tsx 中抽取，避免每次上游同步覆盖 ProviderForm.tsx 时丢失。
 * 所有 visionModel 相关的 state、preset、submit 逻辑集中于此。
 *
 * 用法:
 *   const visionModel = useVisionModel({ initialData, claudeModel });
 *   visionModel.visionModelResolved  → 传给 ClaudeFormFields/CodexFormFields
 *   visionModel.getMeta()            → 在 submit 时合并到 meta
 *   visionModel.setFromPreset(preset) → 选择预设时调用
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function useVisionModel({
  initialData,
  claudeModel,
}: {
  initialData?: any;
  claudeModel?: string;
}) {
  const getInitial = () => (initialData?.meta?.visionModel as string) ?? "";

  const [visionModel, setVisionModel] = useState<string>(getInitial);

  useEffect(() => {
    setVisionModel((initialData?.meta?.visionModel as string) ?? "");
  }, [initialData]);

  const visionModelResolved = visionModel || claudeModel || "";

  const setFromPreset = useCallback(
    (preset: { visionModel?: string }) => {
      setVisionModel(preset.visionModel ?? "");
    },
    [],
  );

  const getMeta = useCallback(() => {
    return { visionModel: visionModel || undefined };
  }, [visionModel]);

  return {
    visionModel,
    setVisionModel,
    visionModelResolved,
    setFromPreset,
    getMeta,
  };
}
