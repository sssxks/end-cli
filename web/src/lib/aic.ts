import { parse as parseToml, stringify as stringifyToml } from 'smol-toml';
import { asInt, asRecord, asString } from './coercions';
import type { AicDraft, PriceRow, Objective, Outpost, Power, Region } from './types';

function asNonNegativeNumber(value: unknown, fallback: number): number {
  const parsed = typeof value === 'number' ? value : Number(value);
  if (!Number.isFinite(parsed) || parsed < 0) {
    return fallback;
  }
  return parsed;
}

function parseRegion(value: unknown): Region {
  const raw = typeof value === 'string' ? value.trim() : '';
  if (raw === 'fourth_valley' || raw === 'wuling') {
    return raw;
  }
  return 'wuling';
}

function parseItemFlowRows(map: Record<string, unknown>): { itemKey: string; value: number }[] {
  return Object.entries(map)
    .map(([itemKey, value]) => ({
      itemKey,
      value: asNonNegativeNumber(value, 0)
    }))
    .filter((row) => row.itemKey.trim().length > 0)
    .sort((a, b) => a.itemKey.localeCompare(b.itemKey));
}

function parseFacilityMachineMaxRows(
  map: Record<string, unknown>
): { facilityKey: string; value: number }[] {
  return Object.entries(map)
    .map(([facilityKey, value]) => ({
      facilityKey,
      value: asInt(value)
    }))
    .filter((row) => row.facilityKey.trim().length > 0)
    .sort((a, b) => a.facilityKey.localeCompare(b.facilityKey));
}

function parsePrices(map: Record<string, unknown>): PriceRow[] {
  return Object.entries(map)
    .map(([itemKey, value]) => ({
      itemKey,
      price: asInt(value)
    }))
    .filter((row) => row.itemKey.trim().length > 0)
    .sort((a, b) => a.itemKey.localeCompare(b.itemKey));
}

function parseOutpost(raw: unknown): Outpost {
  const record = asRecord(raw);
  const zh = asString(record.zh).trim();
  const en = asString(record.en).trim();
  const name = asString(record.name).trim();
  return {
    key: asString(record.key),
    name: name || zh || en,
    moneyCapPerHour: asInt(record.money_cap_per_hour),
    prices: parsePrices(asRecord(record.prices))
  };
}

function parseLegacyObjective(raw: unknown): Objective {
  const record = asRecord(raw);
  const objective = asString(record.objective).trim();
  const alpha = asNonNegativeNumber(record.alpha, 1);
  const beta = asNonNegativeNumber(record.beta, 1);
  const gamma = asNonNegativeNumber(record.gamma, 1);

  if (objective === 'max_power_slack') {
    return { minMachines: 0, maxPowerSlack: 1, maxMoneySlack: 0 };
  }
  if (objective === 'max_money_slack') {
    return { minMachines: 0, maxPowerSlack: 0, maxMoneySlack: 1 };
  }
  if (objective === 'weighted') {
    return { minMachines: alpha, maxPowerSlack: beta, maxMoneySlack: gamma };
  }
  return { minMachines: 1, maxPowerSlack: 0, maxMoneySlack: 0 };
}

function parseObjective(raw: unknown, legacyStage2Raw: unknown): Objective {
  const record = asRecord(raw);
  const hasAny =
    Object.prototype.hasOwnProperty.call(record, 'min_machines') ||
    Object.prototype.hasOwnProperty.call(record, 'max_power_slack') ||
    Object.prototype.hasOwnProperty.call(record, 'max_money_slack');

  if (!hasAny) {
    return parseLegacyObjective(legacyStage2Raw);
  }

  return {
    minMachines: asNonNegativeNumber(record.min_machines, 0),
    maxPowerSlack: asNonNegativeNumber(record.max_power_slack, 0),
    maxMoneySlack: asNonNegativeNumber(record.max_money_slack, 0)
  };
}

function parsePower(raw: unknown, legacyExternalConsumptionRaw: unknown): Power {
  const record = asRecord(raw);
  const hasPowerSection = Object.keys(record).length > 0;
  const enabled = hasPowerSection
    ? Boolean(record.enabled ?? true)
    : true;

  const aliasValue = asInt(record.enternal_consumption);
  const primaryValue = asInt(record.external_consumption);
  const externalConsumptionW = hasPowerSection
    ? (Object.prototype.hasOwnProperty.call(record, 'external_consumption')
        ? primaryValue
        : Object.prototype.hasOwnProperty.call(record, 'enternal_consumption')
          ? aliasValue
          : 0)
    : asInt(legacyExternalConsumptionRaw);

  const externalProductionW = hasPowerSection
    ? (Object.prototype.hasOwnProperty.call(record, 'external_production')
        ? asInt(record.external_production)
        : 200)
    : 200;

  return {
    enabled,
    externalProductionW,
    externalConsumptionW
  };
}

function cleanDraft(draft: AicDraft): AicDraft {
  return {
    region: draft.region === 'fourth_valley' ? 'fourth_valley' : 'wuling',
    power: {
      enabled: Boolean(draft.power.enabled),
      externalProductionW: asInt(draft.power.externalProductionW),
      externalConsumptionW: asInt(draft.power.externalConsumptionW)
    },
    objective: {
      minMachines: asNonNegativeNumber(draft.objective.minMachines, 0),
      maxPowerSlack: asNonNegativeNumber(draft.objective.maxPowerSlack, 0),
      maxMoneySlack: asNonNegativeNumber(draft.objective.maxMoneySlack, 0)
    },
    supply: draft.supply
      .filter((row) => row.itemKey.trim().length > 0)
      .map((row) => ({
        itemKey: row.itemKey.trim(),
        value: asNonNegativeNumber(row.value, 0),
      })),
    consumption: draft.consumption
      .filter((row) => row.itemKey.trim().length > 0)
      .map((row) => ({
        itemKey: row.itemKey.trim(),
        value: asNonNegativeNumber(row.value, 0),
      })),
    facilityMachinesMax: draft.facilityMachinesMax
      .filter((row) => row.facilityKey.trim().length > 0)
      .map((row) => ({
        facilityKey: row.facilityKey.trim(),
        value: asInt(row.value),
      })),
    outposts: draft.outposts
      .filter((outpost) => outpost.key.trim().length > 0)
      .map((outpost) => ({
        key: outpost.key.trim(),
        name: outpost.name.trim(),
        moneyCapPerHour: asInt(outpost.moneyCapPerHour),
        prices: outpost.prices
          .filter((row) => row.itemKey.trim().length > 0)
          .map((row) => ({ itemKey: row.itemKey.trim(), price: asInt(row.price) }))
      }))
  };
}

export function parseAicToml(tomlText: string): AicDraft {
  const parsed = parseToml(tomlText) as Record<string, unknown>;
  return cleanDraft({
    region: parseRegion(parsed.region),
    power: parsePower(parsed.power, parsed.external_power_consumption_w),
    objective: parseObjective(parsed.objective, parsed.stage2),
    supply: parseItemFlowRows(asRecord(parsed.supply_per_min)),
    consumption: parseItemFlowRows(asRecord(parsed.external_consumption_per_min)),
    facilityMachinesMax: parseFacilityMachineMaxRows(asRecord(parsed.facility_machines_max)),
    outposts: Array.isArray(parsed.outposts) ? parsed.outposts.map(parseOutpost) : []
  });
}

export function buildAicToml(draft: AicDraft): string {
  const cleaned = cleanDraft(draft);

  const supplyPerMin = Object.fromEntries(
    cleaned.supply
      .filter((row) => row.value > 0)
      .map((row) => [row.itemKey, asNonNegativeNumber(row.value, 0)])
  );
  const externalConsumptionPerMin = Object.fromEntries(
    cleaned.consumption
      .filter((row) => row.value > 0)
      .map((row) => [row.itemKey, asNonNegativeNumber(row.value, 0)])
  );

  const facilityMachinesMax = Object.fromEntries(
    cleaned.facilityMachinesMax.map((row) => [row.facilityKey, asInt(row.value)])
  );

  const outposts = cleaned.outposts.map((outpost) => {
    const prices = Object.fromEntries(outpost.prices.map((row) => [row.itemKey, asInt(row.price)]));

    const base: Record<string, unknown> = {
      key: outpost.key,
      money_cap_per_hour: asInt(outpost.moneyCapPerHour),
      prices
    };

    if (outpost.name.length > 0) {
      base.zh = outpost.name;
    }

    return base;
  });

  const objective: Record<string, unknown> = {};
  if (cleaned.objective.minMachines > 0) {
    objective.min_machines = cleaned.objective.minMachines;
  }
  if (cleaned.objective.maxPowerSlack > 0) {
    objective.max_power_slack = cleaned.objective.maxPowerSlack;
  }
  if (cleaned.objective.maxMoneySlack > 0) {
    objective.max_money_slack = cleaned.objective.maxMoneySlack;
  }

  const power: Record<string, unknown> = {
    enabled: cleaned.power.enabled
  };
  if (cleaned.power.enabled) {
    power.external_production = asInt(cleaned.power.externalProductionW);
    power.external_consumption = asInt(cleaned.power.externalConsumptionW);
  }

  const root: Record<string, unknown> = {
    version: 2,
    region: cleaned.region,
    power,
    supply_per_min: supplyPerMin,
    external_consumption_per_min: externalConsumptionPerMin,
    facility_machines_max: facilityMachinesMax,
    outposts
  };

  if (Object.keys(objective).length > 0) {
    root.objective = objective;
  }

  if (Object.keys(facilityMachinesMax).length === 0) {
    delete root.facility_machines_max;
  }

  return stringifyToml(root);
}
