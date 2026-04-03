#![allow(clippy::unwrap_used, clippy::expect_used)]

use end_model::{
    AicInputs, Catalog, DisplayName, FacilityDef, FacilityRegions, FacilityU32Map, ItemDef, Key,
    OutpostInput, PosF64, PowerConfig, Region, Stack, Stage2Weights, ThermalBankDef,
};
use end_opt::{Error, NEAR_INT_EPS, run_two_stage};
use generativity::Guard;
use generativity::make_guard;
use std::num::NonZeroU32;

fn key(value: &str) -> Key {
    value.try_into().expect("valid key")
}

fn name(value: &str) -> DisplayName {
    value.try_into().expect("valid display name")
}

fn nz(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("non-zero")
}

fn sample_catalog<'id>(
    guard: Guard<'id>,
    with_recipes: bool,
) -> (Catalog<'id>, end_model::ItemId<'id>, end_model::ItemId<'id>) {
    let mut b = Catalog::builder(guard);
    let ore = b
        .add_item(ItemDef {
            key: key("Ore"),
            en: name("Ore"),
            zh: name("Ore_zh"),
            is_fluid: false,
        })
        .expect("add ore");
    let ingot = b
        .add_item(ItemDef {
            key: key("Ingot"),
            en: name("Ingot"),
            zh: name("Ingot_zh"),
            is_fluid: false,
        })
        .expect("add ingot");

    let machine = b
        .add_facility(FacilityDef {
            key: key("Smelter"),
            power_w: nz(10),
            en: name("Smelter"),
            zh: name("Smelter_zh"),
            regions: FacilityRegions::All,
        })
        .expect("add machine");
    let mut b = b
        .add_thermal_bank(ThermalBankDef {
            key: key("Thermal Bank"),
            en: name("Thermal Bank"),
            zh: name("Thermal_Bank_zh"),
        })
        .expect("add thermal bank");

    if with_recipes {
        b.push_recipe(
            machine,
            nz(60),
            vec![Stack {
                item: ore,
                count: nz(1),
            }]
            .into(),
            vec![Stack {
                item: ingot,
                count: nz(1),
            }]
            .into(),
        )
        .expect("push recipe");
    }

    let catalog = b.build();
    (catalog, ore, ingot)
}

#[test]
fn run_two_stage_applies_region_facility_restrictions() {
    make_guard!(guard);
    let mut b = Catalog::builder(guard);
    let ore = b
        .add_item(ItemDef {
            key: key("Ore"),
            en: name("Ore"),
            zh: name("Ore_zh"),
            is_fluid: false,
        })
        .expect("add ore");
    let ingot = b
        .add_item(ItemDef {
            key: key("Ingot"),
            en: name("Ingot"),
            zh: name("Ingot_zh"),
            is_fluid: false,
        })
        .expect("add ingot");

    let valley_machine = b
        .add_facility(FacilityDef {
            key: key("ValleySmelter"),
            power_w: nz(10),
            en: name("ValleySmelter"),
            zh: name("ValleySmelter_zh"),
            regions: FacilityRegions::FourthValleyOnly,
        })
        .expect("add valley machine");
    let _wuling_machine = b
        .add_facility(FacilityDef {
            key: key("WulingSmelter"),
            power_w: nz(10),
            en: name("WulingSmelter"),
            zh: name("WulingSmelter_zh"),
            regions: FacilityRegions::WulingOnly,
        })
        .expect("add wuling machine");

    let mut b = b
        .add_thermal_bank(ThermalBankDef {
            key: key("Thermal Bank"),
            en: name("Thermal Bank"),
            zh: name("Thermal_Bank_zh"),
        })
        .expect("add thermal bank");

    let valley_recipe = b
        .push_recipe(
            valley_machine,
            nz(60),
            vec![Stack {
                item: ore,
                count: nz(1),
            }]
            .into(),
            vec![Stack {
                item: ingot,
                count: nz(1),
            }]
            .into(),
        )
        .expect("add valley recipe");

    let catalog = b.build();

    make_guard!(aic_guard);
    let mut aic_builder = AicInputs::builder(
        aic_guard,
        PowerConfig::default(),
        vec![(ore, PosF64::new(10.0).expect("positive"))].into(),
        Default::default(),
    )
    .region(Region::FourthValley);
    aic_builder
        .add_outpost(OutpostInput {
            key: key("Camp"),
            en: Some(name("Camp")),
            zh: Some(name("Camp_zh")),
            money_cap_per_hour: 600,
            prices: vec![(ingot, 5)].into(),
        })
        .expect("valid aic outpost");
    let aic = aic_builder.build();

    make_guard!(result_guard);
    let result = run_two_stage(&catalog, &aic, result_guard).expect("solve sample model");

    assert!(
        result
            .stage2
            .recipes_used
            .iter()
            .all(|usage| usage.recipe_index == valley_recipe),
        "only valley recipe should be used under fourth_valley region"
    );
}

fn sample_catalog_and_aic<'cid, 'sid>(
    guard: Guard<'cid>,
    aic_guard: Guard<'sid>,
    with_recipes: bool,
) -> (Catalog<'cid>, AicInputs<'cid, 'sid>) {
    let (catalog, ore, ingot) = sample_catalog(guard, with_recipes);

    let mut aic_builder = AicInputs::builder(
        aic_guard,
        PowerConfig::default(),
        vec![(ore, PosF64::new(10.0).expect("positive"))].into(),
        Default::default(),
    );
    aic_builder
        .add_outpost(OutpostInput {
            key: key("Camp"),
            en: Some(name("Camp")),
            zh: Some(name("Camp_zh")),
            money_cap_per_hour: 600,
            prices: vec![(ingot, 5)].into(),
        })
        .expect("valid aic outpost");
    let aic = aic_builder.build();

    (catalog, aic)
}

#[test]
fn run_two_stage_allows_empty_recipes_with_direct_external_sales() {
    make_guard!(guard);
    let (catalog, ore, _ingot) = sample_catalog(guard, false);
    make_guard!(aic_guard);
    let mut aic_builder = AicInputs::builder(
        aic_guard,
        PowerConfig::default(),
        vec![(ore, PosF64::new(10.0).expect("positive"))].into(),
        Default::default(),
    );
    aic_builder
        .add_outpost(OutpostInput {
            key: key("Camp"),
            en: Some(name("Camp")),
            zh: Some(name("Camp_zh")),
            money_cap_per_hour: 600,
            prices: vec![(ore, 2)].into(),
        })
        .expect("valid aic outpost");
    let aic = aic_builder.build();

    make_guard!(result_guard);
    let result = run_two_stage(&catalog, &aic, result_guard)
        .expect("empty recipes with direct sales should solve");

    assert!(
        (result.stage1.revenue_per_min - 10.0).abs() <= 1e-9,
        "stage1 revenue should be capped at 10/min by outpost cap, got {}",
        result.stage1.revenue_per_min
    );
    let floor = (result.stage1.revenue_per_min
        - NEAR_INT_EPS * result.stage1.revenue_per_min.max(1.0))
    .max(0.0);
    assert!(
        result.stage2.revenue_per_min + 1e-7 >= floor,
        "stage2 revenue {} is lower than floor {}",
        result.stage2.revenue_per_min,
        floor
    );
    assert_eq!(
        result.stage1.total_machines, 0,
        "stage1 should use no machines"
    );
    assert_eq!(
        result.stage2.total_machines, 0,
        "stage2 should use no machines"
    );
    assert!(
        result.stage1.recipes_used.is_empty(),
        "stage1 should report no recipe usage"
    );
    assert!(
        result.stage2.recipes_used.is_empty(),
        "stage2 should report no recipe usage"
    );
    assert!(
        !result.stage1.outpost_sales_qty.is_empty(),
        "stage1 should report non-empty direct sales qty"
    );
}

#[test]
fn run_two_stage_respects_facility_machines_max_constraint() {
    make_guard!(guard);
    let mut b = Catalog::builder(guard);
    let ore = b
        .add_item(ItemDef {
            key: key("Ore"),
            en: name("Ore"),
            zh: name("Ore_zh"),
            is_fluid: false,
        })
        .expect("add ore");
    let ingot = b
        .add_item(ItemDef {
            key: key("Ingot"),
            en: name("Ingot"),
            zh: name("Ingot_zh"),
            is_fluid: false,
        })
        .expect("add ingot");

    let smelter = b
        .add_facility(FacilityDef {
            key: key("Smelter"),
            power_w: nz(10),
            en: name("Smelter"),
            zh: name("Smelter_zh"),
            regions: FacilityRegions::All,
        })
        .expect("add machine");

    let mut b = b
        .add_thermal_bank(ThermalBankDef {
            key: key("Thermal Bank"),
            en: name("Thermal Bank"),
            zh: name("Thermal_Bank_zh"),
        })
        .expect("add thermal bank");

    b.push_recipe(
        smelter,
        nz(60),
        vec![Stack {
            item: ore,
            count: nz(1),
        }]
        .into(),
        vec![Stack {
            item: ingot,
            count: nz(1),
        }]
        .into(),
    )
    .expect("push recipe");

    let catalog = b.build();

    make_guard!(aic_guard);
    let facility_machines_max: FacilityU32Map<'_> = vec![(smelter, 1)].into();
    let mut aic_builder = AicInputs::builder(
        aic_guard,
        PowerConfig::default(),
        vec![(ore, PosF64::new(10.0).expect("positive"))].into(),
        Default::default(),
    )
    .facility_machines_max(facility_machines_max);
    aic_builder
        .add_outpost(OutpostInput {
            key: key("Camp"),
            en: Some(name("Camp")),
            zh: Some(name("Camp_zh")),
            money_cap_per_hour: 600,
            prices: vec![(ingot, 5)].into(),
        })
        .expect("valid aic outpost");
    let aic = aic_builder.build();

    make_guard!(result_guard);
    let result = run_two_stage(&catalog, &aic, result_guard).expect("solve sample model");

    assert_eq!(
        result.stage1.total_machines, 1,
        "stage1 should respect facility machine cap"
    );
    assert_eq!(
        result.stage2.total_machines, 1,
        "stage2 should respect facility machine cap"
    );
    assert!(
        (result.stage1.revenue_per_min - 5.0).abs() <= 1e-9,
        "stage1 revenue should be capped at 1 ingot/min by facility cap, got {}",
        result.stage1.revenue_per_min
    );
}

#[test]
fn max_money_slack_does_not_virtualize_fluid_surplus_into_stockpile() {
    make_guard!(guard);
    let mut b = Catalog::builder(guard);
    let ore = b
        .add_item(ItemDef {
            key: key("Ore"),
            en: name("Ore"),
            zh: name("Ore_zh"),
            is_fluid: false,
        })
        .expect("add ore");
    let coolant = b
        .add_item(ItemDef {
            key: key("Coolant"),
            en: name("Coolant"),
            zh: name("Coolant_zh"),
            is_fluid: true,
        })
        .expect("add coolant");
    let mixer = b
        .add_facility(FacilityDef {
            key: key("Mixer"),
            power_w: nz(10),
            en: name("Mixer"),
            zh: name("Mixer_zh"),
            regions: FacilityRegions::All,
        })
        .expect("add mixer");
    let mut b = b
        .add_thermal_bank(ThermalBankDef {
            key: key("Thermal Bank"),
            en: name("Thermal Bank"),
            zh: name("Thermal_Bank_zh"),
        })
        .expect("add thermal bank");
    b.push_recipe(
        mixer,
        nz(60),
        vec![Stack {
            item: ore,
            count: nz(1),
        }]
        .into(),
        vec![Stack {
            item: coolant,
            count: nz(1),
        }]
        .into(),
    )
    .expect("push recipe");
    let catalog = b.build();

    make_guard!(aic_guard);
    let mut aic_builder = AicInputs::builder(
        aic_guard,
        PowerConfig::default(),
        vec![(ore, PosF64::new(10.0).expect("positive"))].into(),
        Default::default(),
    )
    .stage2_weights(Stage2Weights {
        min_machines: 0.0,
        max_power_slack: 0.0,
        max_money_slack: 1.0,
    });
    aic_builder
        .add_outpost(OutpostInput {
            key: key("Camp"),
            en: Some(name("Camp")),
            zh: Some(name("Camp_zh")),
            money_cap_per_hour: 300,
            prices: vec![(coolant, 1)].into(),
        })
        .expect("valid aic outpost");
    let aic = aic_builder.build();

    make_guard!(result_guard);
    let result = run_two_stage(&catalog, &aic, result_guard).expect("solve sample model");

    assert!(
        (result.stage1.revenue_per_min - 5.0).abs() <= 1e-9,
        "stage1 revenue should be capped at 5/min by outpost cap, got {}",
        result.stage1.revenue_per_min
    );
    assert!(
        result.stage2.money_slack_per_min.abs() <= 1e-9,
        "fluid items must not contribute virtual money slack, got {}",
        result.stage2.money_slack_per_min
    );
    assert!(
        result
            .stage2
            .item_stockpile
            .iter()
            .all(|row| row.item != coolant),
        "fluid items must not appear in warehouse stockpile"
    );
}

#[test]
fn stage2_respects_revenue_floor_and_basic_invariants() {
    make_guard!(guard);
    make_guard!(aic_guard);
    let (catalog, aic) = sample_catalog_and_aic(guard, aic_guard, true);
    make_guard!(result_guard);
    let result = run_two_stage(&catalog, &aic, result_guard).expect("solve sample model");

    let floor = (result.stage1.revenue_per_min
        - NEAR_INT_EPS * result.stage1.revenue_per_min.max(1.0))
    .max(0.0);
    assert!(
        result.stage2.revenue_per_min + 1e-7 >= floor,
        "stage2 revenue {} is lower than floor {}",
        result.stage2.revenue_per_min,
        floor
    );
    assert!(
        !result.stage2.outpost_sales_qty.is_empty(),
        "stage2 should include sale quantity lines"
    );
    for sale in &result.stage2.outpost_sales_qty {
        assert!(
            sale.qty_per_min.get() > 0.0,
            "sale qty must stay strictly positive"
        );
    }

    assert!(
        result.stage2.recipes_used.len() <= 20,
        "recipes_used must be capped at 20"
    );
    for pair in result.stage2.recipes_used.windows(2) {
        assert!(
            pair[0].machines >= pair[1].machines,
            "recipes_used must be sorted descending by machines"
        );
    }
}

#[test]
fn run_two_stage_rejects_infeasible_external_consumption() {
    make_guard!(guard);
    let (catalog, ore, _ingot) = sample_catalog(guard, false);
    make_guard!(aic_guard);
    let aic = AicInputs::builder(
        aic_guard,
        PowerConfig::default(),
        vec![(ore, PosF64::new(10.0).expect("positive"))].into(),
        vec![(ore, PosF64::new(11.0).expect("positive"))].into(),
    )
    .build();

    make_guard!(result_guard);
    let err =
        run_two_stage(&catalog, &aic, result_guard).expect_err("infeasible scenario should fail");
    assert!(
        matches!(err, Error::Solver { .. }),
        "unexpected error: {err:?}"
    );
}
