<script lang="ts">
  import Dialog from "../components/hover/Dialog.svelte";
  import ErrorToast, {
    type ErrorToastState,
  } from "../components/hover/ErrorToast.svelte";
  import ShareDialog from "../components/hover/ShareDialog.svelte";
  import { onMount } from "svelte";
  import HomeDesktopLayout from "../components/layout/HomeDesktopLayout.svelte";
  import HomeMobileLayout from "../components/layout/HomeMobileLayout.svelte";
  import { buildAicToml, parseAicToml } from "../lib/aic";
  import type { FlowSnapshot } from "../lib/export/flow-snapshot";
  import { decodeTomlFromShareParam } from "../lib/share-link";
  import {
    isSameOutpostSelection,
    NO_OUTPOST_SELECTED,
    type OutpostSelection,
  } from "../lib/outpost-selection";
  import {
    addConsumptionRow,
    addFacilityMachinesMaxRow,
    addOutpost,
    addPriceRow,
    addSupplyRow,
    normalizeSelectedOutpostIndex,
    removeConsumptionRow,
    removeFacilityMachinesMaxRow,
    removeOutpost,
    removePriceRow,
    removeSupplyRow,
    setConsumptionKey,
    setConsumptionValue,
    setFacilityMachinesMaxKey,
    setFacilityMachinesMaxValue,
    setObjectiveWeight,
    setPowerEnabled,
    setPowerExternalConsumption,
    setPowerExternalProduction,
    setRegion,
    setOutpostField,
    setPriceKey,
    setPriceValue,
    setSupplyKey,
    setSupplyValue,
  } from "../lib/draft-actions";
  import {
    persistDraft,
    restoreLocalState,
    type DraftStorageConfig,
  } from "../lib/draft-storage";
  import type { EditorActions } from "../lib/editor-actions";
  import {
    createSolverController,
    type SolverController,
  } from "../lib/solver-controller";
  import { renderedOkState, type SolveState } from "../lib/solve-state";
  import { translateByLang } from "../lib/lang";
  import type { AicDraft, CatalogFacilityDto, CatalogItemDto, LangTag } from "../lib/types";
  import { EMPTY_DRAFT } from "../lib/types";
  import { createHydrationPersistGate as createHydrationGate } from "../lib/hydration-persist-gate.svelte";
  import { createBackend, type Backend } from "../lib/backend";
  import bundledDefaultAicToml from "../../../crates/end_io/src/aic.toml?raw";

  let backend: Backend | null = null;
  const backendReady: Promise<Backend> = createBackend().then((nextBackend) => {
    backend = nextBackend;
    return nextBackend;
  });

  const NARROW_LAYOUT_QUERY = "(max-width: 760px)";
  const MIN_EDITOR_WIDTH_PX = 300;
  const MIN_RIGHT_WIDTH_PX = 420;
  const MIN_TOP_PANEL_HEIGHT_PX = 51;
  const MIN_BOTTOM_PANEL_HEIGHT_PX = 51 + 12;
  const AUTO_SOLVE_DEBOUNCE_MS = 200;

  const STORAGE_CONFIG: DraftStorageConfig = {
    draftStorageKey: "end2.web.draft.v2",
  };

  interface Props {
    lang: LangTag;
  }

  let { lang }: Props = $props();
  let catalogItems = $state<CatalogItemDto[]>([]);
  let catalogFacilities = $state<CatalogFacilityDto[]>([]);
  let draft = $state<AicDraft>(structuredClone(EMPTY_DRAFT));
  const defaultToml = bundledDefaultAicToml;

  let isBootstrapping = $state(true);
  let solveState = $state<SolveState>({ status: "idle" });

  let errorToast = $state<ErrorToastState>({ kind: "closed" });

  let selectedOutpostIndex = $state<OutpostSelection>(NO_OUTPOST_SELECTED);
  let isNarrowScreen = $state(false);

  const persistGate = createHydrationGate(() => {
    persistDraft(STORAGE_CONFIG.draftStorageKey, draft);
  });

  const solverController: SolverController = createSolverController({
    debounceMs: AUTO_SOLVE_DEBOUNCE_MS,
    toToml: buildAicToml,
    solve: async (solveLang, toml) => {
      const nextBackend = backend ?? await backendReady;
      return nextBackend.solveScenario(solveLang, toml);
    },
    onStateChange: (next) => {
      solveState = next;
    },
    timeoutApi: {
      setTimeout: globalThis.setTimeout.bind(globalThis),
      clearTimeout: globalThis.clearTimeout.bind(globalThis),
    },
  });

  let isShareDialogOpen = $state(false);
  let shareTomlText = $state("");
  let shareOutputJsonText = $state("");
  let flowSnapshot = $state<FlowSnapshot | null>(null);

  function t(zh: string, en: string): string {
    return translateByLang(lang, zh, en);
  }

  function showErrorToast(message: string): void {
    const trimmed = message.trim();
    if (trimmed.length === 0) {
      return;
    }
    errorToast = { kind: "open", message: trimmed };
  }

  function closeErrorToast(): void {
    errorToast = { kind: "closed" };
  }

  function closeShareDialog(): void {
    isShareDialogOpen = false;
  }

  function openShareDialog(snapshot: FlowSnapshot | null): void {
    try {
      shareTomlText = buildAicToml(draft);
    } catch (error) {
      showErrorToast(error instanceof Error ? error.message : String(error));
      return;
    }

    const ok = renderedOkState(solveState);
    shareOutputJsonText = ok ? JSON.stringify(ok.payload, null, 2) : "";
    flowSnapshot = snapshot;
    isShareDialogOpen = true;
  }

  function applyDraft(nextDraft: AicDraft): void {
    draft = nextDraft;
    selectedOutpostIndex =
      nextDraft.outposts.length > 0
        ? { kind: "selected", index: 0 }
        : NO_OUTPOST_SELECTED;
  }

  function applyToml(text: string): void {
    try {
      const nextDraft = parseAicToml(text);
      applyDraft(nextDraft);
      solverController.resetSolvedFingerprint();
    } catch (error) {
      showErrorToast(error instanceof Error ? error.message : String(error));
    }
  }

  async function resolveSharedDraft(
    shareParam: string | null,
  ): Promise<AicDraft | null> {
    const normalized = shareParam?.trim();
    if (!normalized) {
      return null;
    }

    try {
      const sharedToml = await decodeTomlFromShareParam(normalized);
      return parseAicToml(sharedToml);
    } catch (error) {
      showErrorToast(error instanceof Error ? error.message : String(error));
      return null;
    }
  }

  function isTomlFile(file: File): boolean {
    return file.name.trim().toLowerCase().endsWith(".toml");
  }

  async function importTomlFile(file: File): Promise<void> {
    if (!isTomlFile(file)) {
      showErrorToast(
        t(
          "仅支持导入 *.toml 格式文件。",
          "Only *.toml files are supported for import.",
        ),
      );
      return;
    }

    try {
      const text = await file.text();
      applyToml(text);
    } catch (error) {
      showErrorToast(error instanceof Error ? error.message : String(error));
    }
  }

  async function loadBootstrapData(): Promise<void> {
    isBootstrapping = true;

    try {
      const nextBackend = backend ?? await backendReady;
      const payload = await nextBackend.loadBootstrap(lang);
      catalogItems = payload.catalog.items;
      catalogFacilities = payload.catalog.facilities;
    } catch (error) {
      showErrorToast(error instanceof Error ? error.message : String(error));
    } finally {
      isBootstrapping = false;
    }
  }

  function resetToDefault(): void {
    applyToml(defaultToml);
  }

  function requestResetToDefault(): void {
    isResetDialogOpen = true;
  }

  let isResetDialogOpen = $state(false);

  function closeResetDialog(): void {
    isResetDialogOpen = false;
  }

  async function confirmResetToDefault(): Promise<void> {
    closeResetDialog();
    resetToDefault();
  }

  async function importFromFile(event: Event): Promise<void> {
    const input = event.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) {
      return;
    }

    try {
      await importTomlFile(file);
    } finally {
      input.value = "";
    }
  }

  function exportToml(): void {
    try {
      const toml = buildAicToml(draft);
      const blob = new Blob([toml], { type: "text/plain;charset=utf-8" });
      const url = URL.createObjectURL(blob);
      const anchor = document.createElement("a");
      anchor.href = url;
      anchor.download = "aic.toml";
      anchor.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      showErrorToast(error instanceof Error ? error.message : String(error));
    }
  }

  const editorActions: EditorActions = {
    resetToDefault: requestResetToDefault,
    importFromFile,
    exportToml,
    setRegion: (region) => {
      draft = setRegion(draft, region);
    },
    setPowerEnabled: (enabled) => {
      draft = setPowerEnabled(draft, enabled);
    },
    setPowerExternalProduction: (value) => {
      draft = setPowerExternalProduction(draft, value);
    },
    setPowerExternalConsumption: (value) => {
      draft = setPowerExternalConsumption(draft, value);
    },
    setObjectiveWeight: (field, value) => {
      draft = setObjectiveWeight(draft, field, value);
    },
    supply: {
      add: () => {
        draft = addSupplyRow(draft, catalogItems[0]?.key ?? "");
      },
      remove: (index) => {
        draft = removeSupplyRow(draft, index);
      },
      setKey: (index, key) => {
        draft = setSupplyKey(draft, index, key);
      },
      setValue: (index, value) => {
        draft = setSupplyValue(draft, index, value);
      },
    },
    consumption: {
      add: () => {
        draft = addConsumptionRow(draft, catalogItems[0]?.key ?? "");
      },
      remove: (index) => {
        draft = removeConsumptionRow(draft, index);
      },
      setKey: (index, key) => {
        draft = setConsumptionKey(draft, index, key);
      },
      setValue: (index, value) => {
        draft = setConsumptionValue(draft, index, value);
      },
    },
    facilityMachinesMax: {
      add: () => {
        draft = addFacilityMachinesMaxRow(draft, catalogFacilities[0]?.key ?? "");
      },
      remove: (index) => {
        draft = removeFacilityMachinesMaxRow(draft, index);
      },
      setKey: (index, key) => {
        draft = setFacilityMachinesMaxKey(draft, index, key);
      },
      setValue: (index, value) => {
        draft = setFacilityMachinesMaxValue(draft, index, value);
      },
    },
    outposts: {
      add: () => {
        const next = addOutpost(draft);
        draft = next.draft;
        selectedOutpostIndex = next.selectedOutpostIndex;
      },
      remove: (index) => {
        const next = removeOutpost(draft, selectedOutpostIndex, index);
        draft = next.draft;
        selectedOutpostIndex = next.selectedOutpostIndex;
      },
      select: (index) => {
        selectedOutpostIndex = { kind: "selected", index };
      },
      setField: (index, field, value) => {
        draft = setOutpostField(draft, index, field, value);
      },
    },
    prices: {
      add: (outpostIndex) => {
        draft = addPriceRow(draft, outpostIndex, catalogItems[0]?.key ?? "");
      },
      remove: (outpostIndex, priceIndex) => {
        draft = removePriceRow(draft, outpostIndex, priceIndex);
      },
      setKey: (outpostIndex, priceIndex, key) => {
        draft = setPriceKey(draft, outpostIndex, priceIndex, key);
      },
      setValue: (outpostIndex, priceIndex, value) => {
        draft = setPriceValue(draft, outpostIndex, priceIndex, value);
      },
    },
  };

  onMount(() => {
    let disposed = false;
    let mediaQuery: MediaQueryList | null = null;

    const updateScreenMode = (): void => {
      if (!mediaQuery) {
        return;
      }
      isNarrowScreen = mediaQuery.matches;
    };

    void (async () => {
      void backendReady.then((nextBackend) => nextBackend.warmup()).catch(() => undefined);

      const restored = restoreLocalState(STORAGE_CONFIG);
      const shareParam = new URLSearchParams(window.location.search).get("s");
      const sharedDraft =
        shareParam && shareParam.trim().length > 0
          ? await resolveSharedDraft(shareParam)
          : null;
      const initialDraft = sharedDraft ?? restored.draft;

      if (initialDraft) {
        applyDraft(initialDraft);
      } else {
        applyToml(defaultToml);
      }

      if (disposed) {
        return;
      }

      persistGate.markHydrated();

      mediaQuery = window.matchMedia(NARROW_LAYOUT_QUERY);
      updateScreenMode();
      mediaQuery.addEventListener("change", updateScreenMode);
      void loadBootstrapData();
    })();

    return () => {
      disposed = true;
      mediaQuery?.removeEventListener("change", updateScreenMode);
      solverController.dispose();
    };
  });

  $effect(() => {
    solverController.updateSnapshot({ draft, lang, isBootstrapping });
  });

  $effect(() => {
    persistGate.persistWhenHydrated();
  });

  $effect(() => {
    const normalized = normalizeSelectedOutpostIndex(
      draft,
      selectedOutpostIndex,
    );
    if (!isSameOutpostSelection(normalized, selectedOutpostIndex)) {
      selectedOutpostIndex = normalized;
    }
  });
</script>

{#if isNarrowScreen}
  <HomeMobileLayout
    {lang}
    {draft}
    {catalogItems}
    {catalogFacilities}
    {selectedOutpostIndex}
    {isBootstrapping}
    {solveState}
    {editorActions}
    onOpenShare={openShareDialog}
    onImportFile={importTomlFile}
  />
{:else}
  <HomeDesktopLayout
    {lang}
    {draft}
    {catalogItems}
    {catalogFacilities}
    {selectedOutpostIndex}
    {isBootstrapping}
    {solveState}
    {editorActions}
    onOpenShare={openShareDialog}
    onImportFile={importTomlFile}
    minEditorWidthPx={MIN_EDITOR_WIDTH_PX}
    minRightWidthPx={MIN_RIGHT_WIDTH_PX}
    minTopPanelHeightPx={MIN_TOP_PANEL_HEIGHT_PX}
    minBottomPanelHeightPx={MIN_BOTTOM_PANEL_HEIGHT_PX}
  />
{/if}

<ShareDialog
  open={isShareDialogOpen}
  {lang}
  tomlText={shareTomlText}
  outputJsonText={shareOutputJsonText}
  {flowSnapshot}
  onClose={closeShareDialog}
/>

<ErrorToast
  state={errorToast}
  title={t("错误", "Error")}
  onClose={closeErrorToast}
/>

<Dialog
  open={isResetDialogOpen}
  kind="danger"
  title={t("重置为示例输入", "Reset to Example Input")}
  description={t(
    "重置会覆盖当前所有配置并恢复示例输入，是否继续？",
    "Reset will overwrite current configuration and restore example input. Continue?",
  )}
  cancelLabel={t("取消", "Cancel")}
  confirmLabel={t("重置", "Reset")}
  confirmDisabled={isBootstrapping}
  onCancel={closeResetDialog}
  onConfirm={confirmResetToDefault}
/>
