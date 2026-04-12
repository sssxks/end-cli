import type { AicDraft, CatalogFacilityDto, CatalogItemDto } from './types';
import type { OutpostSelection } from './outpost-selection';

export type OutpostField = 'name' | 'moneyCapPerHour';
export type ObjectiveWeightField = 'minMachines' | 'maxPowerSlack' | 'maxMoneySlack';

export interface EditorActions {
  resetToDefault: () => void;
  importFromFile: (event: Event) => void | Promise<void>;
  exportToml: () => void;
  setRegion: (region: 'fourth_valley' | 'wuling') => void;
  setPowerEnabled: (enabled: boolean) => void;
  setPowerExternalProduction: (value: number) => void;
  setPowerExternalConsumption: (value: number) => void;
  setObjectiveWeight: (field: ObjectiveWeightField, value: number) => void;
  supply: {
    add: () => void;
    remove: (index: number) => void;
    setKey: (index: number, key: string) => void;
    setValue: (index: number, value: number) => void;
  };
  consumption: {
    add: () => void;
    remove: (index: number) => void;
    setKey: (index: number, key: string) => void;
    setValue: (index: number, value: number) => void;
  };
  facilityMachinesMax: {
    add: () => void;
    remove: (index: number) => void;
    setKey: (index: number, key: string) => void;
    setValue: (index: number, value: number) => void;
  };
  outposts: {
    add: () => void;
    remove: (index: number) => void;
    select: (index: number) => void;
    setField: (index: number, field: OutpostField, value: string | number) => void;
  };
  prices: {
    add: (outpostIndex: number) => void;
    remove: (outpostIndex: number, priceIndex: number) => void;
    setKey: (outpostIndex: number, priceIndex: number, key: string) => void;
    setValue: (outpostIndex: number, priceIndex: number, value: number) => void;
  };
}

export interface EditorPanelProps {
  lang: 'zh' | 'en';
  draft: AicDraft;
  catalogItems: CatalogItemDto[];
  catalogFacilities: CatalogFacilityDto[];
  selectedOutpostIndex: OutpostSelection;
  isResetDisabled: boolean;
  actions: EditorActions;
  onOpenShare: () => void;
}
