use end_model::{DisplayName, Key, Region};
use serde::Deserialize;
use serde::de::Error as _;
use serde::de::{Visitor, Unexpected};
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use toml::Spanned;

/// Parsed shape of `items.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ItemsToml {
    pub(crate) items: Box<[Spanned<ItemToml>]>,
}

/// One item entry from `items.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ItemToml {
    #[serde(deserialize_with = "deserialize_key")]
    pub(crate) key: Key,
    #[serde(deserialize_with = "deserialize_display_name")]
    pub(crate) en: DisplayName,
    #[serde(deserialize_with = "deserialize_display_name")]
    pub(crate) zh: DisplayName,
    #[serde(default)]
    pub(crate) fluid: bool,
}

/// Parsed shape of `facilities.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FacilitiesToml {
    #[serde(default)]
    pub(crate) machines: Box<[Spanned<MachineToml>]>,
    pub(crate) thermal_bank: Spanned<ThermalBankToml>,
}

/// One machine facility entry from `facilities.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MachineToml {
    #[serde(deserialize_with = "deserialize_key")]
    pub(crate) key: Key,
    #[serde(deserialize_with = "deserialize_positive_u32")]
    pub(crate) power_w: NonZeroU32,
    #[serde(deserialize_with = "deserialize_display_name")]
    pub(crate) en: DisplayName,
    #[serde(deserialize_with = "deserialize_display_name")]
    pub(crate) zh: DisplayName,
    #[serde(default)]
    pub(crate) regions: Box<[RegionToml]>,
}

/// Thermal bank facility entry from `facilities.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ThermalBankToml {
    #[serde(deserialize_with = "deserialize_key")]
    pub(crate) key: Key,
    #[serde(deserialize_with = "deserialize_display_name")]
    pub(crate) en: DisplayName,
    #[serde(deserialize_with = "deserialize_display_name")]
    pub(crate) zh: DisplayName,
}

/// Parsed shape of `recipes.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipesToml {
    pub(crate) recipes: Box<[Spanned<RecipeToml>]>,
    #[serde(default)]
    pub(crate) power_recipes: Box<[Spanned<PowerRecipeToml>]>,
}

/// Raw `{ item, count }` pair shared by recipe and AIC inputs.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StackToml {
    #[serde(deserialize_with = "deserialize_key")]
    pub(crate) item: Key,
    #[serde(deserialize_with = "deserialize_positive_u32")]
    pub(crate) count: NonZeroU32,
}

/// One machine recipe entry from `recipes.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RecipeToml {
    #[serde(deserialize_with = "deserialize_key")]
    pub(crate) facility: Key,
    #[serde(deserialize_with = "deserialize_positive_u32")]
    pub(crate) time_s: NonZeroU32,
    pub(crate) ingredients: Spanned<Box<[Spanned<StackToml>]>>,
    pub(crate) products: Spanned<Box<[Spanned<StackToml>]>>,
}

/// One power recipe entry from `recipes.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PowerRecipeToml {
    pub(crate) ingredient: Spanned<StackToml>,
    #[serde(deserialize_with = "deserialize_positive_u32")]
    pub(crate) power_w: NonZeroU32,
    #[serde(deserialize_with = "deserialize_positive_u32")]
    pub(crate) time_s: NonZeroU32,
}

/// Parsed shape of `aic.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AicToml {
    #[serde(
        default = "default_aic_version",
        deserialize_with = "deserialize_supported_aic_version"
    )]
    pub(crate) version: u32,
    #[serde(default = "default_region", deserialize_with = "deserialize_region")]
    pub(crate) region: Region,
    #[serde(default)]
    pub(crate) power: PowerToml,
    #[serde(default)]
    pub(crate) objective: ObjectiveToml,
    #[serde(default = "default_empty_spanned_item_positive_f64_map")]
    pub(crate) supply_per_min: Spanned<BTreeMap<KeyToml, PositiveF64Toml>>,
    #[serde(default = "default_empty_spanned_item_positive_f64_map")]
    pub(crate) external_consumption_per_min: Spanned<BTreeMap<KeyToml, PositiveF64Toml>>,
    #[serde(default = "default_empty_spanned_key_non_negative_u32_map")]
    pub(crate) facility_machines_max: Spanned<BTreeMap<KeyToml, NonNegativeU32Toml>>,
    #[serde(default)]
    pub(crate) outposts: Box<[Spanned<OutpostToml>]>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ObjectiveToml {
    pub(crate) min_machines: Option<f64>,
    pub(crate) max_power_slack: Option<f64>,
    pub(crate) max_money_slack: Option<f64>,
}

impl<'de> Deserialize<'de> for ObjectiveToml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawObjectiveToml {
            #[serde(default, deserialize_with = "deserialize_optional_non_negative_f64")]
            min_machines: Option<f64>,
            #[serde(default, deserialize_with = "deserialize_optional_non_negative_f64")]
            max_power_slack: Option<f64>,
            #[serde(default, deserialize_with = "deserialize_optional_non_negative_f64")]
            max_money_slack: Option<f64>,
        }

        let raw = RawObjectiveToml::deserialize(deserializer)?;
        Ok(Self {
            min_machines: raw.min_machines,
            max_power_slack: raw.max_power_slack,
            max_money_slack: raw.max_money_slack,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PowerToml {
    Disabled,
    Enabled {
        external_production_w: u32,
        external_consumption_w: u32,
    },
}

impl Default for PowerToml {
    fn default() -> Self {
        Self::Enabled {
            external_production_w: 200,
            external_consumption_w: 0,
        }
    }
}

impl<'de> Deserialize<'de> for PowerToml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawPowerToml {
            #[serde(default = "default_power_enabled")]
            enabled: bool,
            #[serde(default, deserialize_with = "deserialize_optional_non_negative_u32")]
            external_production: Option<u32>,
            #[serde(default, deserialize_with = "deserialize_optional_non_negative_u32")]
            external_consumption: Option<u32>,
            #[serde(
                default,
                rename = "enternal_consumption",
                deserialize_with = "deserialize_optional_non_negative_u32"
            )]
            enternal_consumption: Option<u32>,
        }

        let raw = RawPowerToml::deserialize(deserializer)?;
        let external_consumption_w = match (raw.external_consumption, raw.enternal_consumption) {
            (Some(value), Some(alias)) if value != alias => {
                return Err(D::Error::custom(
                    "power.external_consumption and power.enternal_consumption conflict",
                ));
            }
            (Some(value), Some(_)) => Some(value),
            (Some(value), None) => Some(value),
            (None, Some(alias)) => Some(alias),
            (None, None) => None,
        };

        if !raw.enabled {
            if raw.external_production.is_some() || external_consumption_w.is_some() {
                return Err(D::Error::custom(
                    "power.external_production/power.external_consumption are not allowed when power.enabled = false",
                ));
            }
            return Ok(Self::Disabled);
        }

        Ok(Self::Enabled {
            external_production_w: raw.external_production.unwrap_or(200),
            external_consumption_w: external_consumption_w.unwrap_or(0),
        })
    }
}

/// One outpost entry from `aic.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OutpostToml {
    #[serde(deserialize_with = "deserialize_key")]
    pub(crate) key: Key,
    #[serde(deserialize_with = "deserialize_non_negative_u32")]
    pub(crate) money_cap_per_hour: u32,
    #[serde(default, deserialize_with = "deserialize_optional_display_name")]
    pub(crate) en: Option<DisplayName>,
    #[serde(default, deserialize_with = "deserialize_optional_display_name")]
    pub(crate) zh: Option<DisplayName>,
    pub(crate) prices: Spanned<BTreeMap<KeyToml, NonNegativeU32Toml>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RegionToml(Region);

impl RegionToml {
    pub(crate) fn into_inner(self) -> Region {
        self.0
    }
}

impl<'de> Deserialize<'de> for RegionToml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        parse_region(raw.as_str())
            .map(Self)
            .map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct KeyToml(Key);

impl KeyToml {
    pub(crate) fn into_inner(self) -> Key {
        self.0
    }
}

impl<'de> Deserialize<'de> for KeyToml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Key::try_from(raw).map(Self).map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PositiveU32Toml(NonZeroU32);

impl PositiveU32Toml {
    pub(crate) fn into_inner(self) -> NonZeroU32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for PositiveU32Toml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i64::deserialize(deserializer)?;
        parse_positive_u32(value)
            .map(Self)
            .map_err(D::Error::custom)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PositiveF64Toml(f64);

impl PositiveF64Toml {
    pub(crate) fn into_inner(self) -> f64 {
        self.0
    }
}

impl<'de> Deserialize<'de> for PositiveF64Toml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct PositiveF64Visitor;

        impl<'de> Visitor<'de> for PositiveF64Visitor {
            type Value = PositiveF64Toml;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a positive number")
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                parse_positive_f64(value as f64).map(PositiveF64Toml).map_err(E::custom)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                parse_positive_f64(value as f64).map(PositiveF64Toml).map_err(E::custom)
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                parse_positive_f64(value).map(PositiveF64Toml).map_err(E::custom)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let parsed = value.parse::<f64>().map_err(|_| {
                    E::invalid_value(Unexpected::Str(value), &"a positive number")
                })?;
                parse_positive_f64(parsed).map(PositiveF64Toml).map_err(E::custom)
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_any(PositiveF64Visitor)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct NonNegativeU32Toml(u32);

impl NonNegativeU32Toml {
    pub(crate) fn into_inner(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for NonNegativeU32Toml {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = i64::deserialize(deserializer)?;
        parse_non_negative_u32(value)
            .map(Self)
            .map_err(D::Error::custom)
    }
}

fn deserialize_key<'de, D>(deserializer: D) -> Result<Key, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    Key::try_from(raw).map_err(D::Error::custom)
}

fn deserialize_display_name<'de, D>(deserializer: D) -> Result<DisplayName, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    DisplayName::try_from(raw).map_err(D::Error::custom)
}

fn deserialize_positive_u32<'de, D>(deserializer: D) -> Result<NonZeroU32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = i64::deserialize(deserializer)?;
    parse_positive_u32(value).map_err(D::Error::custom)
}

fn deserialize_non_negative_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = i64::deserialize(deserializer)?;
    parse_non_negative_u32(value).map_err(D::Error::custom)
}

fn deserialize_optional_non_negative_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<i64>::deserialize(deserializer)?;
    value
        .map(parse_non_negative_u32)
        .transpose()
        .map_err(D::Error::custom)
}

fn deserialize_optional_non_negative_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<f64>::deserialize(deserializer)?;
    value
        .map(parse_non_negative_f64)
        .transpose()
        .map_err(D::Error::custom)
}

fn deserialize_supported_aic_version<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = i64::deserialize(deserializer)?;
    parse_supported_aic_version(value).map_err(D::Error::custom)
}

fn deserialize_optional_display_name<'de, D>(
    deserializer: D,
) -> Result<Option<DisplayName>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = Option::<String>::deserialize(deserializer)?;
    raw.map(|text| DisplayName::try_from(text).map_err(D::Error::custom))
        .transpose()
}

fn parse_region(value: &str) -> Result<Region, String> {
    match value {
        "fourth_valley" => Ok(Region::FourthValley),
        "wuling" => Ok(Region::Wuling),
        other => Err(format!(
            "invalid region `{other}`, expected one of: fourth_valley, wuling"
        )),
    }
}

fn default_region() -> Region {
    Region::FourthValley
}

fn default_aic_version() -> u32 {
    2
}

fn default_power_enabled() -> bool {
    true
}

fn deserialize_region<'de, D>(deserializer: D) -> Result<Region, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    parse_region(raw.as_str()).map_err(D::Error::custom)
}

fn parse_positive_u32(value: i64) -> Result<NonZeroU32, String> {
    if value < 1 {
        return Err(format!("must be >= 1, got {value}"));
    }

    let parsed = u32::try_from(value).map_err(|_| format!("out of range for u32: {value}"))?;
    NonZeroU32::new(parsed).ok_or_else(|| format!("must be >= 1, got {value}"))
}

fn parse_non_negative_u32(value: i64) -> Result<u32, String> {
    if value < 0 {
        return Err(format!("must be >= 0, got {value}"));
    }

    u32::try_from(value).map_err(|_| format!("out of range for u32: {value}"))
}

fn parse_non_negative_f64(value: f64) -> Result<f64, String> {
    if !value.is_finite() {
        return Err("must be a finite number".to_string());
    }
    if value < 0.0 {
        return Err(format!("must be >= 0, got {value}"));
    }
    Ok(value)
}

fn parse_positive_f64(value: f64) -> Result<f64, String> {
    if !value.is_finite() {
        return Err("must be a finite number".to_string());
    }
    if value <= 0.0 {
        return Err(format!("must be > 0, got {value}"));
    }
    Ok(value)
}

fn parse_supported_aic_version(value: i64) -> Result<u32, String> {
    let parsed = parse_non_negative_u32(value)?;
    match parsed {
        2 => Ok(2),
        other => Err(format!("unsupported aic version `{other}`, expected 2")),
    }
}

fn default_empty_spanned_item_positive_f64_map() -> Spanned<BTreeMap<KeyToml, PositiveF64Toml>> {
    Spanned::new(0..0, BTreeMap::new())
}

fn default_empty_spanned_key_non_negative_u32_map() -> Spanned<BTreeMap<KeyToml, NonNegativeU32Toml>> {
    Spanned::new(0..0, BTreeMap::new())
}
