mod builder;
mod types;

pub use builder::AicInputsBuilder;
pub use types::{
    AicBuildError, AicInputs, FacilityU32Map, ItemNonZeroU32Map, ItemPosF64Map, ItemU32Map,
    OutpostId, OutpostInput, PowerConfig, Region, Stage2Weights,
};

impl<'cid, 'sid> AicInputs<'cid, 'sid> {
    pub fn power_config(&self) -> PowerConfig {
        self.power_config
    }

    pub fn stage2_weights(&self) -> Stage2Weights {
        self.stage2_weights
    }

    pub fn region(&self) -> Region {
        self.region
    }

    pub fn supply_per_min(&self) -> &ItemPosF64Map<'cid> {
        &self.supply_per_min
    }

    pub fn external_consumption_per_min(&self) -> &ItemPosF64Map<'cid> {
        &self.external_consumption_per_min
    }

    pub fn facility_machines_max(&self) -> &FacilityU32Map<'cid> {
        &self.facility_machines_max
    }

    pub fn outposts(&self) -> &[OutpostInput<'cid>] {
        &self.outposts
    }

    pub fn outpost(&self, id: OutpostId<'sid>) -> &OutpostInput<'cid> {
        let index = id.index();
        debug_assert!(index < self.outposts.len());
        // SAFETY: `OutpostId<'sid>` is branded by this scenario and is only constructed
        // within this crate from indices into `self.outposts`, so `index` is in-bounds.
        unsafe { self.outposts.get_unchecked(index) }
    }

    pub fn outposts_with_id(
        &self,
    ) -> impl Iterator<Item = (OutpostId<'sid>, &OutpostInput<'cid>)> + '_ {
        let brand = self.scenario_brand;
        self.outposts
            .iter()
            .enumerate()
            .map(move |(index, outpost)| (OutpostId::from_index(index, brand), outpost))
    }
}
