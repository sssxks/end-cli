use std::iter::FromIterator;
use std::num::NonZeroU32;

use generativity::Id;
use thiserror::Error;
use vector_map::VecMap;

use crate::{DisplayName, FacilityId, ItemId, Key, PosF64};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stage2Weights {
    pub min_machines: f64,
    pub max_power_slack: f64,
    pub max_money_slack: f64,
}

impl Default for Stage2Weights {
    fn default() -> Self {
        Self {
            min_machines: 0.0,
            max_power_slack: 0.0,
            max_money_slack: 0.0,
        }
    }
}

impl Stage2Weights {
    pub fn active_target_count(self) -> usize {
        [
            self.min_machines,
            self.max_power_slack,
            self.max_money_slack,
        ]
        .into_iter()
        .filter(|weight| *weight > 0.0)
        .count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerConfig {
    Disabled,
    Enabled {
        external_production_w: u32,
        external_consumption_w: u32,
    },
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self::Enabled {
            external_production_w: 200,
            external_consumption_w: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    FourthValley,
    Wuling,
}

impl Region {
    pub fn as_key(self) -> &'static str {
        match self {
            Self::FourthValley => "fourth_valley",
            Self::Wuling => "wuling",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct OutpostId<'sid> {
    raw: u32,
    brand: Id<'sid>,
}

impl<'sid> OutpostId<'sid> {
    pub fn as_u32(self) -> u32 {
        self.raw
    }

    pub(super) fn from_index(index: usize, brand: Id<'sid>) -> Self {
        Self {
            raw: index as u32,
            brand,
        }
    }

    pub fn index(self) -> usize {
        self.raw as usize
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ItemU32Map<'id>(VecMap<ItemId<'id>, u32>);

impl<'id> ItemU32Map<'id> {
    pub fn new() -> Self {
        Self(VecMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(VecMap::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn insert(&mut self, item: ItemId<'id>, value: u32) -> Option<u32> {
        self.0.insert(item, value)
    }

    pub fn get(&self, item: ItemId<'id>) -> Option<&u32> {
        self.0.get(&item)
    }

    pub fn iter(&self) -> impl Iterator<Item = (ItemId<'id>, u32)> + '_ {
        self.0.iter().map(|(item, value)| (*item, *value))
    }
}

impl<'id> Extend<(ItemId<'id>, u32)> for ItemU32Map<'id> {
    fn extend<T: IntoIterator<Item = (ItemId<'id>, u32)>>(&mut self, iter: T) {
        for (item, value) in iter {
            self.insert(item, value);
        }
    }
}

impl<'id> FromIterator<(ItemId<'id>, u32)> for ItemU32Map<'id> {
    fn from_iter<T: IntoIterator<Item = (ItemId<'id>, u32)>>(iter: T) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl<'id, const N: usize> From<[(ItemId<'id>, u32); N]> for ItemU32Map<'id> {
    fn from(value: [(ItemId<'id>, u32); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<'id> From<Vec<(ItemId<'id>, u32)>> for ItemU32Map<'id> {
    fn from(value: Vec<(ItemId<'id>, u32)>) -> Self {
        value.into_iter().collect()
    }
}

impl<'id> IntoIterator for ItemU32Map<'id> {
    type Item = (ItemId<'id>, u32);
    type IntoIter = vector_map::IntoIter<ItemId<'id>, u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FacilityU32Map<'id>(VecMap<FacilityId<'id>, u32>);

impl<'id> FacilityU32Map<'id> {
    pub fn new() -> Self {
        Self(VecMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(VecMap::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn insert(&mut self, facility: FacilityId<'id>, value: u32) -> Option<u32> {
        self.0.insert(facility, value)
    }

    pub fn get(&self, facility: FacilityId<'id>) -> Option<&u32> {
        self.0.get(&facility)
    }

    pub fn iter(&self) -> impl Iterator<Item = (FacilityId<'id>, u32)> + '_ {
        self.0.iter().map(|(facility, value)| (*facility, *value))
    }
}

impl<'id> Extend<(FacilityId<'id>, u32)> for FacilityU32Map<'id> {
    fn extend<T: IntoIterator<Item = (FacilityId<'id>, u32)>>(&mut self, iter: T) {
        for (facility, value) in iter {
            self.insert(facility, value);
        }
    }
}

impl<'id> FromIterator<(FacilityId<'id>, u32)> for FacilityU32Map<'id> {
    fn from_iter<T: IntoIterator<Item = (FacilityId<'id>, u32)>>(iter: T) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl<'id, const N: usize> From<[(FacilityId<'id>, u32); N]> for FacilityU32Map<'id> {
    fn from(value: [(FacilityId<'id>, u32); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<'id> From<Vec<(FacilityId<'id>, u32)>> for FacilityU32Map<'id> {
    fn from(value: Vec<(FacilityId<'id>, u32)>) -> Self {
        value.into_iter().collect()
    }
}

impl<'id> IntoIterator for FacilityU32Map<'id> {
    type Item = (FacilityId<'id>, u32);
    type IntoIter = vector_map::IntoIter<FacilityId<'id>, u32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ItemNonZeroU32Map<'id>(VecMap<ItemId<'id>, NonZeroU32>);

impl<'id> ItemNonZeroU32Map<'id> {
    pub fn new() -> Self {
        Self(VecMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(VecMap::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn insert(&mut self, item: ItemId<'id>, value: NonZeroU32) -> Option<NonZeroU32> {
        self.0.insert(item, value)
    }

    pub fn get(&self, item: ItemId<'id>) -> Option<&NonZeroU32> {
        self.0.get(&item)
    }

    pub fn iter(&self) -> impl Iterator<Item = (ItemId<'id>, NonZeroU32)> + '_ {
        self.0.iter().map(|(item, value)| (*item, *value))
    }
}

impl<'id> Extend<(ItemId<'id>, NonZeroU32)> for ItemNonZeroU32Map<'id> {
    fn extend<T: IntoIterator<Item = (ItemId<'id>, NonZeroU32)>>(&mut self, iter: T) {
        for (item, value) in iter {
            self.insert(item, value);
        }
    }
}

impl<'id> FromIterator<(ItemId<'id>, NonZeroU32)> for ItemNonZeroU32Map<'id> {
    fn from_iter<T: IntoIterator<Item = (ItemId<'id>, NonZeroU32)>>(iter: T) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl<'id, const N: usize> From<[(ItemId<'id>, NonZeroU32); N]> for ItemNonZeroU32Map<'id> {
    fn from(value: [(ItemId<'id>, NonZeroU32); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<'id> From<Vec<(ItemId<'id>, NonZeroU32)>> for ItemNonZeroU32Map<'id> {
    fn from(value: Vec<(ItemId<'id>, NonZeroU32)>) -> Self {
        value.into_iter().collect()
    }
}

impl<'id> IntoIterator for ItemNonZeroU32Map<'id> {
    type Item = (ItemId<'id>, NonZeroU32);
    type IntoIter = vector_map::IntoIter<ItemId<'id>, NonZeroU32>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ItemPosF64Map<'id>(VecMap<ItemId<'id>, PosF64>);

impl<'id> ItemPosF64Map<'id> {
    pub fn new() -> Self {
        Self(VecMap::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(VecMap::with_capacity(capacity))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn insert(&mut self, item: ItemId<'id>, value: PosF64) -> Option<PosF64> {
        self.0.insert(item, value)
    }

    pub fn get(&self, item: ItemId<'id>) -> Option<&PosF64> {
        self.0.get(&item)
    }

    pub fn iter(&self) -> impl Iterator<Item = (ItemId<'id>, PosF64)> + '_ {
        self.0.iter().map(|(item, value)| (*item, *value))
    }
}

impl<'id> Extend<(ItemId<'id>, PosF64)> for ItemPosF64Map<'id> {
    fn extend<T: IntoIterator<Item = (ItemId<'id>, PosF64)>>(&mut self, iter: T) {
        for (item, value) in iter {
            self.insert(item, value);
        }
    }
}

impl<'id> FromIterator<(ItemId<'id>, PosF64)> for ItemPosF64Map<'id> {
    fn from_iter<T: IntoIterator<Item = (ItemId<'id>, PosF64)>>(iter: T) -> Self {
        let mut map = Self::new();
        map.extend(iter);
        map
    }
}

impl<'id, const N: usize> From<[(ItemId<'id>, PosF64); N]> for ItemPosF64Map<'id> {
    fn from(value: [(ItemId<'id>, PosF64); N]) -> Self {
        value.into_iter().collect()
    }
}

impl<'id> From<Vec<(ItemId<'id>, PosF64)>> for ItemPosF64Map<'id> {
    fn from(value: Vec<(ItemId<'id>, PosF64)>) -> Self {
        value.into_iter().collect()
    }
}

impl<'id> IntoIterator for ItemPosF64Map<'id> {
    type Item = (ItemId<'id>, PosF64);
    type IntoIter = vector_map::IntoIter<ItemId<'id>, PosF64>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct OutpostInput<'id> {
    pub key: Key,
    pub en: Option<DisplayName>,
    pub zh: Option<DisplayName>,
    pub money_cap_per_hour: u32,
    pub prices: ItemU32Map<'id>,
}

#[derive(Debug, Clone)]
pub struct AicInputs<'cid, 'sid> {
    pub(super) region: Region,
    pub(super) supply_per_min: ItemPosF64Map<'cid>,
    pub(super) external_consumption_per_min: ItemPosF64Map<'cid>,
    pub(super) facility_machines_max: FacilityU32Map<'cid>,
    pub(super) outposts: Box<[OutpostInput<'cid>]>,
    pub(super) power_config: PowerConfig,
    pub(super) stage2_weights: Stage2Weights,
    pub(super) scenario_brand: Id<'sid>,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AicBuildError {
    #[error("Duplicate outpost key: {key}")]
    DuplicateOutpostKey { key: Key },
}
