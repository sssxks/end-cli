<script lang="ts">
  import FieldHint from "../hover/FieldHint.svelte";
  import DropdownMenu from "../hover/DropdownMenu.svelte";
  import IconActionButton from "../button/IconActionButton.svelte";
  import InputField from "../input/InputField.svelte";
  import Panel from "../pane/Panel.svelte";
  import PanelHeader from "../pane/PanelHeader.svelte";
  import SelectField from "../input/SelectField.svelte";
  import ToggleSwitch from "../input/ToggleSwitch.svelte";
  import { translateByLang } from "../../lib/lang";
  import type {
    EditorPanelProps,
    ObjectiveWeightField,
  } from "../../lib/editor-actions";
  import type { Outpost } from "../../lib/types";

  type SelectOption = {
    value: string;
    label: string;
  };

  let {
    lang,
    draft,
    catalogItems,
    catalogFacilities,
    selectedOutpostIndex,
    isResetDisabled,
    actions,
    onOpenShare,
  }: EditorPanelProps = $props();

  function t(zh: string, en: string): string {
    return translateByLang(lang, zh, en);
  }

  const selectedIndex = $derived<number | null>(
    selectedOutpostIndex.kind === "selected"
      ? selectedOutpostIndex.index
      : null,
  );
  const selectedOutpost = $derived<Outpost | null>(
    selectedIndex === null ? null : (draft.outposts[selectedIndex] ?? null),
  );
  const selectedOutpostOrdinal = $derived(
    selectedOutpostIndex.kind === "selected"
      ? selectedOutpostIndex.index + 1
      : 1,
  );
  const catalogOptions = $derived<SelectOption[]>(
    catalogItems.map((item) => ({
      value: item.key,
      label: t(item.zh, item.en),
    })),
  );
  const facilityOptions = $derived<SelectOption[]>(
    catalogFacilities.map((facility) => ({
      value: facility.key,
      label: t(facility.zh, facility.en),
    })),
  );
  const regionOptions = $derived<SelectOption[]>([
    { value: "fourth_valley", label: t("四号谷地", "Fourth Valley") },
    { value: "wuling", label: t("武陵", "Wuling") },
  ]);
  const objectiveFieldOrder: ObjectiveWeightField[] = [
    "minMachines",
    "maxPowerSlack",
    "maxMoneySlack",
  ];
  const objectiveFieldOptions = $derived<SelectOption[]>([
    { value: "minMachines", label: t("最少机器", "Min Machines") },
    { value: "maxPowerSlack", label: t("最大电力余量", "Max Power Slack") },
    { value: "maxMoneySlack", label: t("最大虚拟成交额", "Max Money Slack") },
  ]);
  const objectiveRows = $derived.by(() =>
    objectiveFieldOrder
      .map((field) => ({
        field,
        weight: objectiveWeight(field),
      }))
      .filter((row) => row.weight > 0),
  );

  function objectiveWeight(field: ObjectiveWeightField): number {
    if (field === "minMachines") {
      return draft.objective.minMachines;
    }
    if (field === "maxPowerSlack") {
      return draft.objective.maxPowerSlack;
    }
    return draft.objective.maxMoneySlack;
  }

  function objectiveOptionsFor(field: ObjectiveWeightField): SelectOption[] {
    const selected = new Set(objectiveRows.map((row) => row.field));
    return objectiveFieldOptions.filter(
      (option) =>
        option.value === field ||
        !selected.has(option.value as ObjectiveWeightField),
    );
  }

  function addObjectiveTarget(): void {
    const nextField = objectiveFieldOrder.find(
      (field) => objectiveWeight(field) <= 0,
    );
    if (!nextField) {
      return;
    }
    actions.setObjectiveWeight(nextField, 1);
  }

  function removeObjectiveTarget(field: ObjectiveWeightField): void {
    actions.setObjectiveWeight(field, 0);
  }

  function changeObjectiveField(
    currentField: ObjectiveWeightField,
    nextValue: string,
  ): void {
    const nextField = nextValue as ObjectiveWeightField;
    if (!objectiveFieldOrder.includes(nextField)) {
      return;
    }
    if (nextField === currentField) {
      return;
    }
    const weight = objectiveWeight(currentField);
    actions.setObjectiveWeight(currentField, 0);
    actions.setObjectiveWeight(nextField, weight > 0 ? weight : 1);
  }
</script>

<Panel>
  {#snippet header()}
    <PanelHeader
      titleText={t("求解输入", "Solver Inputs")}
      icon="draw"
    >
      {#snippet controls()}
        <IconActionButton
          icon="share"
          onClick={onOpenShare}
          label={t("分享", "Share")}
          ariaLabel={t("分享", "Share")}
        />

        <DropdownMenu
          menuAriaLabel={t("输入操作菜单", "Input actions menu")}
          triggerAriaLabel={t("打开菜单", "Open menu")}
          triggerIcon="more_vert"
          disabled={false}
        >
          {#snippet menu(close)}
            <IconActionButton
              icon="download"
              label={t("导入 aic.toml", "Import aic.toml")}
              ariaLabel={t("导入 aic.toml", "Import aic.toml")}
              fileInput={{
                accept: ".toml,text/plain",
                onChange: (event) => {
                  close();
                  return actions.importFromFile(event);
                },
              }}
            />

            <IconActionButton
              icon="upload"
              label={t("导出 aic.toml", "Export aic.toml")}
              onClick={() => {
                close();
                actions.exportToml();
              }}
              ariaLabel={t("导出 aic.toml", "Export aic.toml")}
            />

            <IconActionButton
              kind="danger"
              icon="delete"
              label={t("重置为示例输入", "Reset to Example Input")}
              onClick={() => {
                close();
                actions.resetToDefault();
              }}
              disabled={isResetDisabled}
              ariaLabel={t("重置示例输入", "Reset Example Input")}
            />
          {/snippet}
        </DropdownMenu>
      {/snippet}
    </PanelHeader>
  {/snippet}

  <section class="editor-shell">
    <div class="field-row">
      <div class="label-with-hint">
        <label for="region">{t("地区", "Region")}</label>
        <FieldHint
          text={t(
            "过滤出只有该区域可用的机器。",
            "Region does not change outpost data; it mainly controls availability for machines with region locks.",
          )}
        />
      </div>
      <SelectField
        id="region"
        value={draft.region}
        options={regionOptions}
        ariaLabel={t("选择地区", "Select region")}
        searchable={false}
        onChange={(nextValue) =>
          actions.setRegion(nextValue as "fourth_valley" | "wuling")}
      />
    </div>

    <section>
      <PanelHeader>
        {#snippet title()}
          <div class="heading-with-hint">
            <h3>{t("优化目标", "Stage-2 Targets")}</h3>
            <FieldHint
              text={t(
                "程序首先会最大化利润，然后在利润最优的解空间内根据这里设置的目标进行二次优化。",
                "The program first maximizes profit, then performs secondary optimization based on the targets set here within the profit-optimal solution space.",
              )}
            />
          </div>
        {/snippet}

        {#snippet controls()}
          <IconActionButton
            icon="add"
            onClick={addObjectiveTarget}
            disabled={objectiveRows.length >= objectiveFieldOrder.length}
            ariaLabel={t("添加目标", "Add stage-2 target")}
          />
        {/snippet}
      </PanelHeader>

      {#if objectiveRows.length === 0}
        <p class="hint">
          {t(
            "暂无优化目标。",
            "No stage-2 target configured; stage-1 optimum will be used directly.",
          )}
        </p>
      {/if}

      {#each objectiveRows as row (row.field)}
        <div class="row-grid objective-row">
          <SelectField
            value={row.field}
            options={objectiveOptionsFor(row.field)}
            ariaLabel={t("选择阶段二目标", "Select stage-2 target")}
            searchable={false}
            onChange={(nextValue) => changeObjectiveField(row.field, nextValue)}
          />

          <InputField
            type="number"
            min="0"
            step="0.1"
            value={row.weight}
            oninput={(event) =>
              actions.setObjectiveWeight(
                row.field,
                Number((event.currentTarget as HTMLInputElement).value),
              )}
          />

          <IconActionButton
            icon="horizontal_rule"
            onClick={() => removeObjectiveTarget(row.field)}
            ariaLabel={t("删除阶段二目标", "Remove stage-2 target")}
          />
        </div>

        {#if !draft.power.enabled && row.field === "maxPowerSlack"}
          <p class="objective-warning">
            {t(
              "警告：电力计算已关闭，此目标权重会被忽略。",
              "Warning: power modeling is disabled, this target weight is ignored.",
            )}
          </p>
        {/if}
      {/each}

      <PanelHeader>
        {#snippet title()}
          <div class="heading-with-hint">
            <h3>{t("电力", "Power")}</h3>
          </div>
        {/snippet}
      </PanelHeader>

      <div class="field-row">
        <div class="label-with-hint">
          <span>{t("电力建模", "Power Modeling")}</span>
          <FieldHint
            text={t(
              "开启后，模型会考虑电力供需平衡，自动计算电池消耗。",
              "When enabled, the model considers power supply/demand balance and automatically accounts for battery consumption.",
            )}
          />
        </div>
        <ToggleSwitch
          id="power-enabled"
          checked={draft.power.enabled}
          label={t("启用电力计算", "Enable power calculations")}
          ariaLabel={t("切换电力计算", "Toggle power calculations")}
          onToggle={(nextValue) => actions.setPowerEnabled(nextValue)}
        />
      </div>

      {#if draft.power.enabled}
        <div class="field-row">
          <div class="label-with-hint">
            <label for="power-external-production"
              >{t("外部发电", "External Production (W)")}</label
            >
            <FieldHint
              text={t(
                "用于建模系统外稳定提供的发电量，默认值为核心赠送的 200W",
                "Stable external generation provided outside modeled production loops; default value is 200W gifted by the core.",
              )}
            />
          </div>
          <InputField
            id="power-external-production"
            type="number"
            min="0"
            value={draft.power.externalProductionW}
            oninput={(event) => {
              actions.setPowerExternalProduction(
                Number((event.currentTarget as HTMLInputElement).value),
              );
            }}
          />
        </div>

        <div class="field-row">
          <div class="label-with-hint">
            <label for="power-external-consumption"
              >{t("外部耗电", "External Consumption (W)")}</label
            >
            <FieldHint
              text={t(
                "用于建模矿点、滑索、作战设备等系统外耗电。",
                "Stable external usage from mining points, ziplines, combat devices, and other off-model systems.",
              )}
            />
          </div>
          <InputField
            id="power-external-consumption"
            type="number"
            min="0"
            value={draft.power.externalConsumptionW}
            oninput={(event) => {
              actions.setPowerExternalConsumption(
                Number((event.currentTarget as HTMLInputElement).value),
              );
            }}
          />
        </div>
      {/if}
    </section>

    <section>
      <PanelHeader>
        {#snippet title()}
          <div class="heading-with-hint">
            <h3>{t("外部供给 / min", "External Supply / min")}</h3>
            <FieldHint
              text={t(
                "通常用于表示矿点持续开采的矿物供给。",
                "Typically used for minerals continuously supplied by mining points.",
              )}
            />
          </div>
        {/snippet}

        {#snippet controls()}
          <IconActionButton
            icon="add"
            onClick={actions.supply.add}
            ariaLabel={t("添加供给条目", "Add supply row")}
          />
        {/snippet}
      </PanelHeader>

      {#if draft.supply.length === 0}
        <p class="hint">{t("暂无供给条目。", "No supply rows yet.")}</p>
      {/if}

      {#each draft.supply as row, rowIndex (rowIndex)}
        <div class="row-grid">
          <SelectField
            value={row.itemKey}
            options={catalogOptions}
            ariaLabel={t("选择供给物品", "Select supply item")}
            searchPlaceholder={t("搜索物品...", "Search items...")}
            emptyText={t("无匹配物品", "No matching items")}
            onChange={(nextValue) => actions.supply.setKey(rowIndex, nextValue)}
          />

          <InputField
            type="number"
            min="0"
            step="any"
            value={row.value}
            oninput={(event) =>
              actions.supply.setValue(
                rowIndex,
                Number((event.currentTarget as HTMLInputElement).value),
              )}
          />

          <IconActionButton
            icon="horizontal_rule"
            onClick={() => actions.supply.remove(rowIndex)}
            ariaLabel={t("删除供给条目", "Remove supply row")}
          />
        </div>
      {/each}
    </section>

    <section>
      <PanelHeader>
        {#snippet title()}
          <div class="heading-with-hint">
            <h3>{t("设备数量", "Facility Machines Max")}</h3>
            <FieldHint
              text={t(
                "限制指定设备在产线中允许使用的最大台数（含对应配方的机器总数）。",
                "Limits the maximum allowed machine count for a given facility across the whole plan.",
              )}
            />
          </div>
        {/snippet}

        {#snippet controls()}
          <IconActionButton
            icon="add"
            onClick={actions.facilityMachinesMax.add}
            ariaLabel={t("添加设备限制条目", "Add facility limit row")}
          />
        {/snippet}
      </PanelHeader>

      {#if draft.facilityMachinesMax.length === 0}
        <p class="hint">{t("暂无设备限制条目。", "No facility limit rows yet.")}</p>
      {/if}

      {#each draft.facilityMachinesMax as row, rowIndex (rowIndex)}
        <div class="row-grid">
          <SelectField
            value={row.facilityKey}
            options={facilityOptions}
            ariaLabel={t("选择设备", "Select facility")}
            searchPlaceholder={t("搜索设备...", "Search facilities...")}
            emptyText={t("无匹配设备", "No matching facilities")}
            onChange={(nextValue) =>
              actions.facilityMachinesMax.setKey(rowIndex, nextValue)}
          />

          <InputField
            type="number"
            min="0"
            step="1"
            value={row.value}
            oninput={(event) =>
              actions.facilityMachinesMax.setValue(
                rowIndex,
                Number((event.currentTarget as HTMLInputElement).value),
              )}
          />

          <IconActionButton
            icon="horizontal_rule"
            onClick={() => actions.facilityMachinesMax.remove(rowIndex)}
            ariaLabel={t("删除设备限制条目", "Remove facility limit row")}
          />
        </div>
      {/each}
    </section>

    <section>
      <PanelHeader>
        {#snippet title()}
          <div class="heading-with-hint">
            <h3>{t("外部消耗 / min", "External Consumption / min")}</h3>
            <FieldHint
              text={t(
                "通常用于表示日用品和快递货物等外部需求，须填写每分钟平均数据。",
                "Typically used for stable external demand such as daily supplies and delivery cargo, converted to per-minute averages.",
              )}
            />
          </div>
        {/snippet}

        {#snippet controls()}
          <IconActionButton
            icon="add"
            onClick={actions.consumption.add}
            ariaLabel={t("添加消耗条目", "Add consumption row")}
          />
        {/snippet}
      </PanelHeader>

      {#if draft.consumption.length === 0}
        <p class="hint">{t("暂无消耗条目。", "No consumption rows yet.")}</p>
      {/if}

      {#each draft.consumption as row, rowIndex (rowIndex)}
        <div class="row-grid">
          <SelectField
            value={row.itemKey}
            options={catalogOptions}
            ariaLabel={t("选择消耗物品", "Select consumption item")}
            searchPlaceholder={t("搜索物品...", "Search items...")}
            emptyText={t("无匹配物品", "No matching items")}
            onChange={(nextValue) =>
              actions.consumption.setKey(rowIndex, nextValue)}
          />

          <InputField
            type="number"
            min="0"
            step="any"
            value={row.value}
            oninput={(event) =>
              actions.consumption.setValue(
                rowIndex,
                Number((event.currentTarget as HTMLInputElement).value),
              )}
          />

          <IconActionButton
            icon="horizontal_rule"
            onClick={() => actions.consumption.remove(rowIndex)}
            ariaLabel={t("删除消耗条目", "Remove consumption row")}
          />
        </div>
      {/each}
    </section>

    <section>
      <PanelHeader>
        {#snippet title()}
          <div class="heading-with-hint">
            <h3>{t("据点", "Outposts")}</h3>
            <FieldHint
              text={t(
                "此列表可以留空；填入后求解器会根据收购价和供需情况自动计算生产什么产品来交易以最大化利润。",
                "This section can be left empty for the solver to ignore outposts; when filled, the solver automatically calculates which products to produce for trading based on their prices and supply/demand to maximize profit.",
              )}
            />
          </div>
        {/snippet}

        {#snippet controls()}
          <IconActionButton
            icon="add"
            onClick={actions.outposts.add}
            ariaLabel={t("添加据点", "Add outpost")}
          />
        {/snippet}
      </PanelHeader>

      {#if draft.outposts.length === 0}
        <p class="hint">{t("暂无据点。", "No outposts yet.")}</p>
      {/if}

      {#if draft.outposts.length > 0}
        <div class="outpost-layout">
          <div class="outpost-list">
            {#each draft.outposts as outpost, outpostIndex (outpost.key)}
              <button
                type="button"
                class={`outpost-pick ${selectedOutpostIndex.kind === "selected" && outpostIndex === selectedOutpostIndex.index ? "active" : ""}`}
                onclick={() => actions.outposts.select(outpostIndex)}
              >
                <p class="outpost-pick-title">
                  {outpost.name ||
                    outpost.key ||
                    `${t("据点", "Outpost")} ${outpostIndex + 1}`}
                </p>
                <p class="outpost-pick-meta">
                  {t("交易上限", "Cap / h")}: {outpost.moneyCapPerHour}
                </p>
                <p class="outpost-pick-meta">
                  {t("收购条目", "Price rows")}: {outpost.prices.length}
                </p>
              </button>
            {/each}
          </div>

          {#if selectedOutpost}
            <article class="outpost-card">
              <div class="outpost-head">
                <h4>
                  {selectedOutpost.name ||
                    selectedOutpost.key ||
                    `${t("据点", "Outpost")} ${selectedOutpostOrdinal}`}
                </h4>
                <IconActionButton
                  icon="delete"
                  onClick={() => {
                    if (selectedOutpostIndex.kind !== "selected") {
                      return;
                    }
                    actions.outposts.remove(selectedOutpostIndex.index);
                  }}
                  ariaLabel={t("删除据点", "Remove outpost")}
                />
              </div>

              <div class="row-grid two">
                <label>
                  {t("名称", "Name (Optional)")}
                  <InputField
                    type="text"
                    value={selectedOutpost.name}
                    oninput={(event) => {
                      if (selectedOutpostIndex.kind !== "selected") {
                        return;
                      }
                      actions.outposts.setField(
                        selectedOutpostIndex.index,
                        "name",
                        (event.currentTarget as HTMLInputElement).value,
                      );
                    }}
                  />
                </label>

                <label>
                  <span class="label-with-hint">
                    <span>{t("每小时交易上限", "Money Cap / h")}</span>
                    <FieldHint
                      text={t(
                        "据点每小时可用于交易的金额上限。",
                        "Maximum amount of money this outpost can trade per hour.",
                      )}
                    />
                  </span>
                  <InputField
                    type="number"
                    min="0"
                    value={selectedOutpost.moneyCapPerHour}
                    oninput={(event) => {
                      if (selectedOutpostIndex.kind !== "selected") {
                        return;
                      }
                      actions.outposts.setField(
                        selectedOutpostIndex.index,
                        "moneyCapPerHour",
                        Number((event.currentTarget as HTMLInputElement).value),
                      );
                    }}
                  />
                </label>
              </div>

              <PanelHeader>
                {#snippet title()}
                  <div class="heading-with-hint">
                    <h5>{t("收购价", "Buy Prices")}</h5>
                    <FieldHint
                      text={t(
                        "除按游戏内价格填写外，可手动删去低价且容易爆仓的条目来收缩优化范围。本程序使用稳态模型，不会主动考虑爆仓风险。",
                        "Besides filling from in-game prices, you can remove low-value, overflow-prone rows to narrow optimization scope. This program uses a steady-state model and does not actively consider overflow risk.",
                      )}
                    />
                  </div>
                {/snippet}

                {#snippet controls()}
                  <IconActionButton
                    icon="add"
                    onClick={() => {
                      if (selectedOutpostIndex.kind !== "selected") {
                        return;
                      }
                      actions.prices.add(selectedOutpostIndex.index);
                    }}
                    ariaLabel={t("添加价格条目", "Add price row")}
                  />
                {/snippet}
              </PanelHeader>

              {#if selectedOutpost.prices.length === 0}
                <p class="hint">{t("暂无价格条目。", "No price rows yet.")}</p>
              {/if}

              {#each selectedOutpost.prices as price, priceIndex (priceIndex)}
                <div class="row-grid">
                  <SelectField
                    value={price.itemKey}
                    options={catalogOptions}
                    ariaLabel={t("选择收购物品", "Select price item")}
                    searchPlaceholder={t("搜索物品...", "Search items...")}
                    emptyText={t("无匹配物品", "No matching items")}
                    onChange={(nextValue) => {
                      if (selectedOutpostIndex.kind !== "selected") {
                        return;
                      }
                      actions.prices.setKey(
                        selectedOutpostIndex.index,
                        priceIndex,
                        nextValue,
                      );
                    }}
                  />

                  <InputField
                    type="number"
                    min="0"
                    value={price.price}
                    oninput={(event) => {
                      if (selectedOutpostIndex.kind !== "selected") {
                        return;
                      }
                      actions.prices.setValue(
                        selectedOutpostIndex.index,
                        priceIndex,
                        Number((event.currentTarget as HTMLInputElement).value),
                      );
                    }}
                  />

                  <IconActionButton
                    icon="horizontal_rule"
                    onClick={() => {
                      if (selectedOutpostIndex.kind !== "selected") {
                        return;
                      }
                      actions.prices.remove(
                        selectedOutpostIndex.index,
                        priceIndex,
                      );
                    }}
                    ariaLabel={t("删除价格条目", "Remove price row")}
                  />
                </div>
              {/each}
            </article>
          {/if}
        </div>
      {/if}
    </section>
  </section>
</Panel>

<style>
  .editor-shell {
    display: grid;
    gap: var(--space-3);
    container-type: inline-size;
  }

  .editor-shell > section {
    display: grid;
    gap: var(--space-2);
  }

  .editor-shell h3 {
    margin: 0;
    line-height: 1.2;
  }

  .field-row {
    display: grid;
    grid-template-columns: 1fr 190px;
    gap: var(--space-2);
    align-items: center;
  }

  .label-with-hint {
    display: inline-flex;
    align-items: center;
  }

  .heading-with-hint {
    display: inline-flex;
    align-items: center;
  }

  .objective-row {
    margin-top: 0;
  }

  .objective-warning {
    margin: calc(var(--space-1) * -1) 0 0;
    color: var(--danger);
    font-size: 12px;
    line-height: 1.35;
  }

  .outpost-pick {
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    background: var(--panel-strong);
    display: grid;
    gap: var(--space-1);
    text-align: left;
    padding: var(--space-3);
  }

  .outpost-pick.active {
    border-color: color-mix(in srgb, var(--accent) 58%, var(--accent-tint-1));
    box-shadow: inset 0 0 0 2px
      color-mix(in srgb, var(--accent) 35%, var(--accent-tint-2));
  }

  .outpost-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: var(--space-2);
  }

  .outpost-pick-title {
    font-size: 14px;
    font-weight: 600;
    margin: 0;
  }

  .outpost-pick-meta {
    margin: 0;
    color: var(--ink-soft);
    font-size: 12px;
  }

  .outpost-card {
    border: 1px solid var(--line);
    border-radius: var(--radius-md);
    padding: var(--space-3);
    display: grid;
    gap: var(--space-2);
    background: var(--panel-strong);
    container-type: inline-size;
  }

  .row-grid.two label {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .row-grid {
    display: grid;
    gap: var(--space-2);
    grid-template-columns: 4fr minmax(64px, 1fr) 36px;
  }

  .row-grid > * {
    min-width: 0;
  }

  .row-grid.two {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .outpost-layout {
    display: grid;
    gap: var(--space-3);
  }

  .outpost-list {
    display: grid;
    gap: var(--space-2);
    align-content: start;
  }

  @container (min-width: 480px) {
    .outpost-layout {
      grid-template-columns: minmax(140px, 1fr) minmax(0, 3fr);
    }
  }

  @container (max-width: 480px) {
    .outpost-layout {
      grid-template-columns: 1fr;
    }

    .outpost-list {
      grid-template-columns: 1fr 1fr;
    }
  }

  @container (max-width: 280px) {
    .row-grid.two {
      grid-template-columns: 1fr;
    }
  }
</style>
