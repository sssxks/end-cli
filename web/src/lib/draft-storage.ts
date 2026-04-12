import { asInt, asRecord, asString } from './coercions';
import { localStorageGet, localStorageRemove, localStorageSet } from './storage';
import type { AicDraft } from './types';

export interface DraftStorageConfig {
  draftStorageKey: string;
}

export interface RestoredLocalState {
  draft: AicDraft | null;
}

function parseStoredDraft(text: string): AicDraft | null {
  try {
    const root = JSON.parse(text);
    if (typeof root !== 'object' || root === null || Array.isArray(root)) {
      return null;
    }

    const parsed = root as Record<string, unknown>;
    const regionRaw = asString(parsed.region).trim();
    const region = regionRaw === 'fourth_valley' ? 'fourth_valley' : 'wuling';
    const supplyRows = Array.isArray(parsed.supply) ? parsed.supply : [];
    const consumptionRows = Array.isArray(parsed.consumption) ? parsed.consumption : [];
    const facilityRows = Array.isArray(parsed.facilityMachinesMax) ? parsed.facilityMachinesMax : [];
    const outpostRows = Array.isArray(parsed.outposts) ? parsed.outposts : [];
    const asNonNegativeNumber = (value: unknown): number => {
      const parsedNumber = typeof value === 'number' ? value : Number(value);
      if (!Number.isFinite(parsedNumber) || parsedNumber < 0) {
        return 0;
      }
      return parsedNumber;
    };
    const powerRecord = asRecord(parsed.power);
    const objectiveRecord = asRecord(parsed.objective);
    const legacyStage2Record = asRecord(parsed.stage2);

    const hasPower =
      Object.prototype.hasOwnProperty.call(powerRecord, 'enabled') ||
      Object.prototype.hasOwnProperty.call(powerRecord, 'externalProductionW') ||
      Object.prototype.hasOwnProperty.call(powerRecord, 'externalConsumptionW');
    const power = hasPower
      ? {
          enabled: Boolean(powerRecord.enabled ?? true),
          externalProductionW: asInt(powerRecord.externalProductionW),
          externalConsumptionW: asInt(powerRecord.externalConsumptionW)
        }
      : {
          enabled: true,
          externalProductionW: 200,
          externalConsumptionW: asInt(parsed.externalPowerConsumptionW)
        };

    const hasObjective =
      Object.prototype.hasOwnProperty.call(objectiveRecord, 'minMachines') ||
      Object.prototype.hasOwnProperty.call(objectiveRecord, 'maxPowerSlack') ||
      Object.prototype.hasOwnProperty.call(objectiveRecord, 'maxMoneySlack');

    const objective = hasObjective
      ? {
          minMachines: asNonNegativeNumber(objectiveRecord.minMachines),
          maxPowerSlack: asNonNegativeNumber(objectiveRecord.maxPowerSlack),
          maxMoneySlack: asNonNegativeNumber(objectiveRecord.maxMoneySlack)
        }
      : (() => {
          const legacyObjective = asString(legacyStage2Record.objective).trim();
          if (legacyObjective === 'max_power_slack') {
            return { minMachines: 0, maxPowerSlack: 1, maxMoneySlack: 0 };
          }
          if (legacyObjective === 'max_money_slack') {
            return { minMachines: 0, maxPowerSlack: 0, maxMoneySlack: 1 };
          }
          if (legacyObjective === 'weighted') {
            return {
              minMachines: asNonNegativeNumber(legacyStage2Record.alpha),
              maxPowerSlack: asNonNegativeNumber(legacyStage2Record.beta),
              maxMoneySlack: asNonNegativeNumber(legacyStage2Record.gamma)
            };
          }
          return { minMachines: 1, maxPowerSlack: 0, maxMoneySlack: 0 };
        })();

    return {
      region,
      power,
      objective,
      supply: supplyRows.map((row) => {
        const record = asRecord(row);
        return {
          itemKey: asString(record.itemKey),
          value: asNonNegativeNumber(record.value)
        };
      }),
      consumption: consumptionRows.map((row) => {
        const record = asRecord(row);
        return {
          itemKey: asString(record.itemKey),
          value: asNonNegativeNumber(record.value)
        };
      }),
      facilityMachinesMax: facilityRows.map((row) => {
        const record = asRecord(row);
        return {
          facilityKey: asString(record.facilityKey),
          value: asInt(record.value)
        };
      }),
      outposts: outpostRows.map((row) => {
        const record = asRecord(row);
        const priceRows = Array.isArray(record.prices) ? record.prices : [];
        const zh = asString(record.zh).trim();
        const en = asString(record.en).trim();
        const name = asString(record.name).trim();
        return {
          key: asString(record.key),
          name: name || zh || en,
          moneyCapPerHour: asInt(record.moneyCapPerHour),
          prices: priceRows.map((priceRow) => {
            const priceRecord = asRecord(priceRow);
            return {
              itemKey: asString(priceRecord.itemKey),
              price: asInt(priceRecord.price)
            };
          })
        };
      })
    };
  } catch {
    return null;
  }
}

export function restoreLocalState(config: DraftStorageConfig): RestoredLocalState {
  const storedDraft = localStorageGet(config.draftStorageKey);
  if (storedDraft === null) {
    return {
      draft: null
    };
  }

  const draft = parseStoredDraft(storedDraft);
  if (!draft) {
    localStorageRemove(config.draftStorageKey);
  }

  return {
    draft
  };
}

export function persistDraft(storageKey: string, draft: AicDraft): void {
  localStorageSet(storageKey, JSON.stringify(draft));
}

export function persistLeftPaneRatio(storageKey: string, ratio: number): void {
  persistPaneRatio(storageKey, ratio);
}

export function persistRightPaneRatio(storageKey: string, ratio: number): void {
  persistPaneRatio(storageKey, ratio);
}

function persistPaneRatio(storageKey: string, ratio: number): void {
  localStorageSet(storageKey, String(ratio));
}
