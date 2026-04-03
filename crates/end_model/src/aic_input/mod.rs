mod model;

pub use model::{
    AicBuildError, AicInputs, AicInputsBuilder, FacilityU32Map, ItemNonZeroU32Map, ItemPosF64Map,
    ItemU32Map, OutpostId, OutpostInput, PowerConfig, Region, Stage2Weights,
};

#[cfg(test)]
mod tests;
