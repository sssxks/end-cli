import { describe, expect, it } from 'vitest';
import { buildAicToml, parseAicToml } from './aic';
import type { AicDraft } from './types';

describe('aic toml conversions', () => {
  it('round-trips draft through TOML while preserving normalized data', () => {
    const draft: AicDraft = {
      region: 'fourth_valley',
      power: {
        enabled: true,
        externalProductionW: 350,
        externalConsumptionW: 322
      },
      objective: {
        minMachines: 1.5,
        maxPowerSlack: 2,
        maxMoneySlack: 0.5
      },
      supply: [
        { itemKey: 'IronOre', value: 120 },
        { itemKey: 'CopperOre', value: 80 }
      ],
      consumption: [
        { itemKey: 'IronOre', value: 15 },
        { itemKey: 'Water', value: 8 }
      ],
      facilityMachinesMax: [
        { facilityKey: 'Smelter', value: 0 },
        { facilityKey: 'Assembler', value: 12 }
      ],
      outposts: [
        {
          key: 'Refugee_Camp',
          name: 'Refugee Camp',
          moneyCapPerHour: 17316,
          prices: [
            { itemKey: 'SC Valley Battery', price: 30 },
            { itemKey: 'Origocrust', price: 1 }
          ]
        }
      ]
    };

    const toml = buildAicToml(draft);
    const parsed = parseAicToml(toml);

    expect(parsed.power.externalProductionW).toBe(350);
    expect(parsed.power.externalConsumptionW).toBe(322);
    expect(parsed.region).toBe('fourth_valley');
    expect(parsed.objective.minMachines).toBe(1.5);
    expect(parsed.objective.maxPowerSlack).toBe(2);
    expect(parsed.objective.maxMoneySlack).toBe(0.5);
    expect(parsed.supply).toHaveLength(2);
    expect(parsed.consumption).toHaveLength(2);
    expect(parsed.facilityMachinesMax).toHaveLength(2);
    expect(parsed.outposts).toHaveLength(1);
    expect(parsed.outposts[0]?.key).toBe('Refugee_Camp');
    expect(parsed.outposts[0]?.name).toBe('Refugee Camp');
    expect(parsed.outposts[0]?.prices).toHaveLength(2);
  });

  it('preserves fractional supply/consumption flows when building and parsing TOML', () => {
    const draft: AicDraft = {
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
      supply: [
        { itemKey: 'IronOre', value: 1.5 },
        { itemKey: 'CopperOre', value: 0.25 }
      ],
      consumption: [{ itemKey: 'Water', value: 2.75 }],
      facilityMachinesMax: [],
      outposts: []
    };

    const toml = buildAicToml(draft);
    const parsed = parseAicToml(toml);

    expect(parsed.supply.find((row) => row.itemKey === 'IronOre')?.value).toBeCloseTo(1.5);
    expect(parsed.supply.find((row) => row.itemKey === 'CopperOre')?.value).toBeCloseTo(0.25);
    expect(parsed.consumption.find((row) => row.itemKey === 'Water')?.value).toBeCloseTo(2.75);
  });

  it('parses legacy en/zh outpost names into unified name field', () => {
    const toml = `version = 2
region = "wuling"

[[outposts]]
key = "Legacy_Outpost"
zh = "旧据点"
en = "Legacy Outpost"
money_cap_per_hour = 10
[outposts.prices]
"Battery" = 2
`;

    const parsed = parseAicToml(toml);

    expect(parsed.outposts).toHaveLength(1);
    expect(parsed.outposts[0]?.name).toBe('旧据点');
  });

  it('drops blank keys during cleaning', () => {
    const toml = `version = 2
region = "wuling"
[power]
enabled = true
external_production = 200
external_consumption = 10
[supply_per_min]
"IronOre" = 5
"" = 99

[external_consumption_per_min]
"IronOre" = 2
"" = 99

[[outposts]]
key = ""
money_cap_per_hour = 100
[outposts.prices]
"Battery" = 2
`;

    const parsed = parseAicToml(toml);

    expect(parsed.supply).toHaveLength(1);
    expect(parsed.consumption).toHaveLength(1);
    expect(parsed.outposts).toHaveLength(0);
    expect(parsed.region).toBe('wuling');
  });

  it('omits disabled objective fields when building toml', () => {
    const draft: AicDraft = {
      region: 'wuling',
      power: {
        enabled: true,
        externalProductionW: 200,
        externalConsumptionW: 10
      },
      objective: {
        minMachines: 0,
        maxPowerSlack: 0,
        maxMoneySlack: 2
      },
      supply: [],
      consumption: [],
      facilityMachinesMax: [],
      outposts: []
    };

    const toml = buildAicToml(draft);

    expect(toml).toContain('[objective]');
    expect(toml).toContain('max_money_slack = 2');
    expect(toml).not.toContain('min_machines =');
    expect(toml).not.toContain('max_power_slack =');
  });

  it('parses legacy stage2 and external power fields', () => {
    const toml = `region = "wuling"
external_power_consumption_w = 10

[stage2]
objective = "weighted"
alpha = 1.5
beta = 2
gamma = 3
`;

    const parsed = parseAicToml(toml);

    expect(parsed.power.enabled).toBe(true);
    expect(parsed.power.externalProductionW).toBe(200);
    expect(parsed.power.externalConsumptionW).toBe(10);
    expect(parsed.objective.minMachines).toBe(1.5);
    expect(parsed.objective.maxPowerSlack).toBe(2);
    expect(parsed.objective.maxMoneySlack).toBe(3);
  });
});
