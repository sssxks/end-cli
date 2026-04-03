<script lang="ts">
  import DragImportOverlay from "../hover/DragImportOverlay.svelte";
  import EditorPanel from "../workbench/EditorPanel.svelte";
  import GraphPanel from "../workbench/GraphPanel.svelte";
  import ResultPanel from "../workbench/ResultPanel.svelte";
  import type { EditorActions } from "../../lib/editor-actions";
  import type { FlowSnapshot } from "../../lib/export/flow-snapshot";
  import { translateByLang } from "../../lib/lang";
  import type { SolveState } from "../../lib/solve-state";
  import type { AicDraft, CatalogFacilityDto, CatalogItemDto, LangTag } from "../../lib/types";
  import type { OutpostSelection } from "../../lib/outpost-selection";

  type MobileTab = "editor" | "result" | "graph";

  interface Props {
    lang: LangTag;
    draft: AicDraft;
    catalogItems: CatalogItemDto[];
    catalogFacilities: CatalogFacilityDto[];
    selectedOutpostIndex: OutpostSelection;
    isBootstrapping: boolean;
    solveState: SolveState;
    editorActions: EditorActions;
    onOpenShare: (snapshot: FlowSnapshot | null) => void;
    onImportFile: (file: File) => void | Promise<void>;
  }

  let {
    lang,
    draft,
    catalogItems,
    catalogFacilities,
    selectedOutpostIndex,
    isBootstrapping,
    solveState,
    editorActions,
    onOpenShare,
    onImportFile,
  }: Props = $props();

  type GraphPanelApi = {
    getFlowSnapshot: () => FlowSnapshot | null;
  };

  let activeTab = $state<MobileTab>("editor");
  let graphPanelApi = $state<GraphPanelApi | null>(null);

  function t(zh: string, en: string): string {
    return translateByLang(lang, zh, en);
  }

  function handleOpenShare(): void {
    onOpenShare(graphPanelApi?.getFlowSnapshot() ?? null);
  }
</script>

<nav class="mobile-tabs" aria-label={t("页面分区", "Panel tabs")}>
  <button
    type="button"
    class:active={activeTab === "editor"}
    onclick={() => (activeTab = "editor")}
  >
    {t("输入", "Inputs")}
  </button>
  <button
    type="button"
    class:active={activeTab === "result"}
    onclick={() => (activeTab = "result")}
  >
    {t("评估", "Summary")}
  </button>
  <button
    type="button"
    class:active={activeTab === "graph"}
    onclick={() => (activeTab = "graph")}
  >
    {t("物流", "Flow")}
  </button>
</nav>

<main class="workspace">
  <section class={`${activeTab !== "editor" ? "tab-hidden" : "editor"}`}>
    <EditorPanel
      {lang}
      {draft}
      {catalogItems}
      {catalogFacilities}
      {selectedOutpostIndex}
      isResetDisabled={isBootstrapping}
      actions={editorActions}
      onOpenShare={handleOpenShare}
    />
  </section>

  <section class={`${activeTab !== "result" ? "tab-hidden" : "result"}`}>
    <ResultPanel {lang} {isBootstrapping} {solveState} />
  </section>

  <section class={`${activeTab !== "graph" ? "tab-hidden" : "graph"}`}>
    <GraphPanel bind:this={graphPanelApi} {lang} {solveState} />
  </section>

  <DragImportOverlay {lang} onImportFile={onImportFile} />
</main>
