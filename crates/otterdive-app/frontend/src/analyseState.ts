export type AnalyseEnterAction = "search" | "update" | "add";

export function resolveAnalyseEnterAction(
  hasSelectedPattern: boolean,
  draftChanged: boolean,
  configuredAction: AnalyseEnterAction,
): AnalyseEnterAction {
  if (!hasSelectedPattern) return "add";
  if (!draftChanged) return "search";
  return configuredAction;
}
