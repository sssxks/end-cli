export type LangTag = 'zh' | 'en';
export type Region = 'fourth_valley' | 'wuling';

export interface Objective {
  minMachines: number;
  maxPowerSlack: number;
  maxMoneySlack: number;
}

export interface Power {
  enabled: boolean;
  externalProductionW: number;
  externalConsumptionW: number;
}

export interface CatalogItemDto {
  key: string;
  en: string;
  zh: string;
}

export interface CatalogFacilityDto {
  key: string;
  en: string;
  zh: string;
}

export interface BootstrapPayload {
  catalog: {
    items: CatalogItemDto[];
    facilities: CatalogFacilityDto[];
  };
}

export interface OutpostValue {
  key: string;
  name: string;
  valuePerMin: number;
  capPerMin: number;
  ratio: number;
}

export interface SaleValue {
  outpostKey: string;
  outpostName: string;
  itemKey: string;
  itemName: string;
  qtyPerMin: number;
  valuePerMin: number;
}

export interface FacilityUsage {
  key: string;
  name: string;
  machines: number;
  powerW: number;
  totalPowerW: number;
}

export interface ExternalSupplySlack {
  itemKey: string;
  itemName: string;
  supplyPerMin: number;
  slackPerMin: number;
}

export interface PowerSummary {
  externalProductionW: number;
  externalConsumptionW: number;
  thermalGenerationW: number;
  machineConsumptionW: number;
  totalGenW: number;
  totalUseW: number;
  marginW: number;
}

export interface Summary {
  lang: LangTag;
  stage1RevenuePerMin: number;
  stage2RevenuePerMin: number;
  stage2RevenuePerHour: number;
  totalMachines: number;
  totalThermalBanks: number;
  power: PowerSummary | null;
  outposts: OutpostValue[];
  topSales: SaleValue[];
  facilities: FacilityUsage[];
  externalSupplySlack: ExternalSupplySlack[];
}

export type LogisticsNodeKind =
  | 'external_supply'
  | 'external_consumption'
  | 'recipe_group'
  | 'outpost_sale'
  | 'thermal_bank_group'
  | 'warehouse_stockpile';

export interface LogisticsItemSummary {
  itemKey: string;
  itemName: string;
  edgeCount: number;
  nodeCount: number;
  totalFlowPerMin: number;
}

export interface LogisticsNode {
  id: string;
  kind: LogisticsNodeKind | string;
  label: string;
}

export interface LogisticsEdge {
  id: string;
  itemKey: string;
  itemName: string;
  source: string;
  target: string;
  flowPerMin: number;
}

export interface LogisticsGraph {
  items: LogisticsItemSummary[];
  nodes: LogisticsNode[];
  edges: LogisticsEdge[];
}

export interface SolvePayload {
  reportText: string;
  summary: Summary;
  logisticsGraph: LogisticsGraph;
}

export interface ApiErrorEnvelope {
  status: 'err';
  error: {
    message: string;
    source?: string;
  };
}

export interface ApiOkEnvelope<T> {
  status: 'ok';
  data: T;
}

export type ApiEnvelope<T> = ApiOkEnvelope<T> | ApiErrorEnvelope;

export interface SupplyRow {
  itemKey: string;
  value: number;
}

export interface ConsumptionRow {
  itemKey: string;
  value: number;
}

export interface FacilityMachinesMaxRow {
  facilityKey: string;
  value: number;
}

export interface PriceRow {
  itemKey: string;
  price: number;
}

export interface Outpost {
  key: string;
  name: string;
  moneyCapPerHour: number;
  prices: PriceRow[];
}

export interface AicDraft {
  region: Region;
  power: Power;
  objective: Objective;
  supply: SupplyRow[];
  consumption: ConsumptionRow[];
  facilityMachinesMax: FacilityMachinesMaxRow[];
  outposts: Outpost[];
}

export const EMPTY_DRAFT: AicDraft = {
  region: 'fourth_valley',
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
