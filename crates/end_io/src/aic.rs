use crate::error::map_aic_build_error;
use crate::schema::{AicToml, PowerToml};
use crate::{Error, Result};
use end_model::{
    AicInputs, Catalog, FacilityU32Map, ItemPosF64Map, ItemU32Map, OutpostInput, PosF64,
    PowerConfig, Stage2Weights,
};
use generativity::{Guard, make_guard};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const BUILTIN_AIC_TOML: &str = include_str!("aic.toml");
const BUILTIN_AIC_PATH: &str = "<builtin>/aic.toml";
const MEMORY_AIC_PATH: &str = "<memory>/aic.toml";

/// Load `aic.toml` from disk and resolve key-based references against a catalog.
pub fn load_aic<'cid, 'sid>(
    path: &Path,
    catalog: &Catalog<'cid>,
    guard: Guard<'sid>,
) -> Result<AicInputs<'cid, 'sid>> {
    let path = path.to_path_buf();
    let src = match std::fs::read_to_string(&path) {
        Ok(src) => src,
        Err(source) => return Err(Error::Io { path, source }),
    };
    let raw: AicToml = match toml::from_str(src.as_str()) {
        Ok(raw) => raw,
        Err(source) => return Err(Error::TomlParse { path, source }),
    };
    resolve_aic(path, Arc::<str>::from(src), raw, catalog, guard)
}

/// Parse `aic.toml` from in-memory text and resolve references against a catalog.
pub fn load_aic_from_str<'cid, 'sid>(
    src: &str,
    catalog: &Catalog<'cid>,
    guard: Guard<'sid>,
) -> Result<AicInputs<'cid, 'sid>> {
    let path = PathBuf::from(MEMORY_AIC_PATH);
    let raw: AicToml = match toml::from_str(src) {
        Ok(raw) => raw,
        Err(source) => return Err(Error::TomlParse { path, source }),
    };
    resolve_aic(path, Arc::<str>::from(src), raw, catalog, guard)
}

/// Serialize [`default_aic`] as pretty TOML.
pub fn default_aic_toml<'id>(catalog: &Catalog<'id>) -> Result<String> {
    // validate first. because user can specify `init --data-dir`, we want to make sure the built-in AIC is valid against the potentially customized catalog.
    let path = PathBuf::from(BUILTIN_AIC_PATH);
    let src = Arc::<str>::from(BUILTIN_AIC_TOML);
    let raw: AicToml = match toml::from_str(BUILTIN_AIC_TOML) {
        Ok(raw) => raw,
        Err(source) => return Err(Error::TomlParse { path, source }),
    };
    make_guard!(guard);
    resolve_aic(path, src, raw, catalog, guard)?;
    Ok(BUILTIN_AIC_TOML.to_string())
}

/// Convert parsed AIC TOML into validated domain inputs and resolve catalog references.
fn resolve_aic<'cid, 'sid>(
    path: PathBuf,
    src: Arc<str>,
    raw: AicToml,
    catalog: &Catalog<'cid>,
    guard: Guard<'sid>,
) -> Result<AicInputs<'cid, 'sid>> {
    let _version = raw.version;
    let region = raw.region;
    let power_config = match raw.power {
        PowerToml::Disabled => PowerConfig::Disabled,
        PowerToml::Enabled {
            external_production_w,
            external_consumption_w,
        } => PowerConfig::Enabled {
            external_production_w,
            external_consumption_w,
        },
    };
    let stage2_weights = Stage2Weights {
        min_machines: raw.objective.min_machines.unwrap_or(0.0),
        max_power_slack: raw.objective.max_power_slack.unwrap_or(0.0),
        max_money_slack: raw.objective.max_money_slack.unwrap_or(0.0),
    };

    if matches!(power_config, PowerConfig::Disabled) && stage2_weights.max_power_slack > 0.0 {
        return Err(Error::Schema {
            path: path.clone(),
            field: "objective.max_power_slack",
            index: None,
            span: None,
            src: Some(Arc::clone(&src)),
            message: "objective.max_power_slack must be 0 when power.enabled = false"
                .to_string()
                .into_boxed_str(),
        });
    }

    let supply_per_min_span = raw.supply_per_min.span();
    let raw_supply_per_min = raw.supply_per_min.into_inner();
    let mut supply_per_min = ItemPosF64Map::with_capacity(raw_supply_per_min.len());
    for (item_key, value) in raw_supply_per_min {
        let item_key = item_key.into_inner();
        let value = value.into_inner();
        let value = PosF64::new(value).ok_or_else(|| Error::Schema {
            path: path.clone(),
            field: "supply_per_min",
            index: None,
            span: Some(supply_per_min_span.clone()),
            src: Some(Arc::clone(&src)),
            message: format!("must be > 0, got {value}").into_boxed_str(),
        })?;
        let item = catalog
            .item_id(item_key.as_str())
            .ok_or_else(|| Error::UnknownItem {
                path: path.clone(),
                key: item_key.into(),
                span: Some(supply_per_min_span.clone()),
                src: Some(Arc::clone(&src)),
            })?;
        supply_per_min.insert(item, value);
    }

    let external_consumption_per_min_span = raw.external_consumption_per_min.span();
    let raw_external_consumption_per_min = raw.external_consumption_per_min.into_inner();
    let mut external_consumption_per_min =
        ItemPosF64Map::with_capacity(raw_external_consumption_per_min.len());
    for (item_key, value) in raw_external_consumption_per_min {
        let item_key = item_key.into_inner();
        let value = value.into_inner();
        let value = PosF64::new(value).ok_or_else(|| Error::Schema {
            path: path.clone(),
            field: "external_consumption_per_min",
            index: None,
            span: Some(external_consumption_per_min_span.clone()),
            src: Some(Arc::clone(&src)),
            message: format!("must be > 0, got {value}").into_boxed_str(),
        })?;
        let item = catalog
            .item_id(item_key.as_str())
            .ok_or_else(|| Error::UnknownItem {
                path: path.clone(),
                key: item_key.into(),
                span: Some(external_consumption_per_min_span.clone()),
                src: Some(Arc::clone(&src)),
            })?;
        external_consumption_per_min.insert(item, value);
    }

    let facility_machines_max_span = raw.facility_machines_max.span();
    let raw_facility_machines_max = raw.facility_machines_max.into_inner();
    let mut facility_machines_max = FacilityU32Map::with_capacity(raw_facility_machines_max.len());
    for (facility_key, value) in raw_facility_machines_max {
        let facility_key = facility_key.into_inner();
        let value = value.into_inner();
        let facility = catalog
            .facility_id(facility_key.as_str())
            .ok_or_else(|| Error::UnknownFacility {
                path: path.clone(),
                key: facility_key.into(),
                span: Some(facility_machines_max_span.clone()),
                src: Some(Arc::clone(&src)),
            })?;
        facility_machines_max.insert(facility, value);
    }

    let mut builder = AicInputs::builder(
        guard,
        power_config,
        supply_per_min,
        external_consumption_per_min,
    )
    .region(region)
    .facility_machines_max(facility_machines_max)
    .stage2_weights(stage2_weights);

    let mut outpost_spans = BTreeMap::new();
    for outpost in raw.outposts {
        let outpost_span = outpost.span();
        let outpost = outpost.into_inner();

        outpost_spans.insert(outpost.key.to_string(), outpost_span.clone());

        let prices_span = outpost.prices.span();
        let raw_prices = outpost.prices.into_inner();
        let mut prices = ItemU32Map::with_capacity(raw_prices.len());
        for (item_key, price) in raw_prices {
            let item_key = item_key.into_inner();
            let price = price.into_inner();

            let item = catalog
                .item_id(item_key.as_str())
                .ok_or_else(|| Error::UnknownItem {
                    path: path.clone(),
                    key: item_key.into(),
                    span: Some(prices_span.clone()),
                    src: Some(Arc::clone(&src)),
                })?;
            prices.insert(item, price);
        }

        builder
            .add_outpost(OutpostInput {
                key: outpost.key,
                en: outpost.en,
                zh: outpost.zh,
                money_cap_per_hour: outpost.money_cap_per_hour,
                prices,
            })
            .map_err(|source| {
                map_aic_build_error(path.clone(), Arc::clone(&src), source, &outpost_spans)
            })?;
    }

    Ok(builder.build())
}
