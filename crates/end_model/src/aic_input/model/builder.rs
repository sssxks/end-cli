use std::collections::HashSet;

use generativity::Guard;

use super::{
    AicBuildError, AicInputs, FacilityU32Map, ItemPosF64Map, OutpostId, OutpostInput, PowerConfig,
    Region, Stage2Weights,
};

#[derive(Debug)]
pub struct AicInputsBuilder<'cid, 'sid> {
    region: Region,
    supply_per_min: ItemPosF64Map<'cid>,
    external_consumption_per_min: ItemPosF64Map<'cid>,
    facility_machines_max: FacilityU32Map<'cid>,
    outposts: Vec<OutpostInput<'cid>>,
    outpost_keys: HashSet<crate::Key>,
    power_config: PowerConfig,
    stage2_weights: Stage2Weights,
    scenario_brand: generativity::Id<'sid>,
}

impl<'cid, 'sid> AicInputs<'cid, 'sid> {
    pub fn builder(
        guard: Guard<'sid>,
        power_config: PowerConfig,
        supply_per_min: ItemPosF64Map<'cid>,
        external_consumption_per_min: ItemPosF64Map<'cid>,
    ) -> AicInputsBuilder<'cid, 'sid> {
        AicInputsBuilder {
            region: Region::FourthValley,
            supply_per_min,
            external_consumption_per_min,
            facility_machines_max: FacilityU32Map::default(),
            outposts: Vec::new(),
            outpost_keys: HashSet::new(),
            power_config,
            stage2_weights: Stage2Weights::default(),
            scenario_brand: guard.into(),
        }
    }
}

impl<'cid, 'sid> AicInputsBuilder<'cid, 'sid> {
    pub fn region(mut self, region: Region) -> Self {
        self.region = region;
        self
    }

    pub fn facility_machines_max(mut self, facility_machines_max: FacilityU32Map<'cid>) -> Self {
        self.facility_machines_max = facility_machines_max;
        self
    }

    pub fn stage2_weights(mut self, stage2_weights: Stage2Weights) -> Self {
        self.stage2_weights = stage2_weights;
        self
    }

    pub fn add_outpost(
        &mut self,
        outpost: OutpostInput<'cid>,
    ) -> Result<OutpostId<'sid>, AicBuildError> {
        if !self.outpost_keys.insert(outpost.key.clone()) {
            return Err(AicBuildError::DuplicateOutpostKey { key: outpost.key });
        }

        let id = OutpostId::from_index(self.outposts.len(), self.scenario_brand);
        self.outposts.push(outpost);
        Ok(id)
    }

    pub fn build(self) -> AicInputs<'cid, 'sid> {
        let Self {
            region,
            supply_per_min,
            external_consumption_per_min,
            facility_machines_max,
            outposts,
            outpost_keys: _,
            power_config,
            stage2_weights,
            scenario_brand,
        } = self;

        AicInputs {
            region,
            supply_per_min,
            external_consumption_per_min,
            facility_machines_max,
            outposts: outposts.into_boxed_slice(),
            power_config,
            stage2_weights,
            scenario_brand,
        }
    }
}
