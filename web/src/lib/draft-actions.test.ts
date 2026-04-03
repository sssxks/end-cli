import { describe, expect, it } from 'vitest';
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
  setConsumptionValue,
  setFacilityMachinesMaxValue,
  setObjectiveWeight,
  setOutpostField,
  setPowerEnabled,
  setPowerExternalConsumption,
  setPowerExternalProduction,
  setPriceValue,
  setSupplyValue
} from './draft-actions';
import type { AicDraft } from './types';

const EMPTY_DRAFT: AicDraft = {
  region: 'wuling',
  power: {
    enabled: true,
    externalProductionW: 200,
    externalConsumptionW: 0
  },
  objective: {
    minMachines: 0,
    maxPowerSlack: 0,
    maxMoneySlack: 0
  },
  supply: [],
  consumption: [],
  facilityMachinesMax: [],
  outposts: []
};

describe('draft actions', () => {
  it('normalizes power/price as non-negative integers while keeping flows as non-negative numbers', () => {
    let draft = setPowerExternalConsumption(EMPTY_DRAFT, -12.3);
    draft = setPowerExternalProduction(draft, 66.2);
    draft = addSupplyRow(draft, 'IronOre');
    draft = setSupplyValue(draft, 0, 4.8);
    draft = addConsumptionRow(draft, 'Water');
    draft = setConsumptionValue(draft, 0, 6.7);

    expect(draft.power.externalConsumptionW).toBe(0);
    expect(draft.power.externalProductionW).toBe(66);
    expect(draft.supply[0]?.value).toBe(4.8);
    expect(draft.consumption[0]?.value).toBe(6.7);
  });

  it('adds and removes outposts while maintaining selected index', () => {
    const added1 = addOutpost(EMPTY_DRAFT);
    const added2 = addOutpost(added1.draft);

    expect(added2.draft.outposts).toHaveLength(2);
    expect(added2.selectedOutpostIndex).toEqual({ kind: 'selected', index: 1 });
    expect(added2.draft.outposts[0]?.key).toBe('outpost_1');
    expect(added2.draft.outposts[1]?.key).toBe('outpost_2');

    const removed = removeOutpost(added2.draft, added2.selectedOutpostIndex, 1);
    expect(removed.draft.outposts).toHaveLength(1);
    expect(removed.selectedOutpostIndex).toEqual({ kind: 'selected', index: 0 });
  });

  it('updates outpost fields and price rows immutably', () => {
    const added = addOutpost(EMPTY_DRAFT);

    let draft = setOutpostField(added.draft, 0, 'name', 'TradeHub');
    draft = addPriceRow(draft, 0, 'Circuit');
    draft = setPriceValue(draft, 0, 0, 12.6);

    expect(draft.outposts[0]?.name).toBe('TradeHub');
    expect(draft.outposts[0]?.key).toBe('outpost_1');
    expect(draft.outposts[0]?.prices[0]?.price).toBe(13);

    const afterRemovePrice = removePriceRow(draft, 0, 0);
    expect(afterRemovePrice.outposts[0]?.prices).toHaveLength(0);
  });

  it('removes supply rows by index', () => {
    let draft = addSupplyRow(EMPTY_DRAFT, 'A');
    draft = addSupplyRow(draft, 'B');

    const after = removeSupplyRow(draft, 0);
    expect(after.supply).toHaveLength(1);
    expect(after.supply[0]?.itemKey).toBe('B');
  });

  it('removes consumption rows by index', () => {
    let draft = addConsumptionRow(EMPTY_DRAFT, 'A');
    draft = addConsumptionRow(draft, 'B');

    const after = removeConsumptionRow(draft, 0);
    expect(after.consumption).toHaveLength(1);
    expect(after.consumption[0]?.itemKey).toBe('B');
  });

  it('adds and removes facility machine max rows by index', () => {
    let draft = addFacilityMachinesMaxRow(EMPTY_DRAFT, 'Smelter');
    draft = setFacilityMachinesMaxValue(draft, 0, 3.7);
    expect(draft.facilityMachinesMax[0]?.facilityKey).toBe('Smelter');
    expect(draft.facilityMachinesMax[0]?.value).toBe(4);

    draft = addFacilityMachinesMaxRow(draft, 'Assembler');
    const after = removeFacilityMachinesMaxRow(draft, 0);
    expect(after.facilityMachinesMax).toHaveLength(1);
    expect(after.facilityMachinesMax[0]?.facilityKey).toBe('Assembler');
  });

  it('normalizes selected outpost index for empty and invalid states', () => {
    expect(normalizeSelectedOutpostIndex(EMPTY_DRAFT, { kind: 'selected', index: 2 })).toEqual({ kind: 'none' });

    const added = addOutpost(EMPTY_DRAFT);
    expect(normalizeSelectedOutpostIndex(added.draft, { kind: 'selected', index: 9 })).toEqual({ kind: 'selected', index: 0 });
  });

  it('updates objective weights and power toggle', () => {
    let draft = setObjectiveWeight(EMPTY_DRAFT, 'minMachines', 1.25);
    draft = setObjectiveWeight(draft, 'maxPowerSlack', 2.5);
    draft = setObjectiveWeight(draft, 'maxMoneySlack', -3);
    draft = setPowerEnabled(draft, false);

    expect(draft.objective.minMachines).toBe(1.25);
    expect(draft.objective.maxPowerSlack).toBe(2.5);
    expect(draft.objective.maxMoneySlack).toBe(0);
    expect(draft.power.enabled).toBe(false);
  });
});
