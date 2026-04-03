use std::collections::{BTreeMap, BTreeSet};

use end_io::{load_aic_from_str, load_catalog};
use end_model::{
    AicInputs, Catalog, FacilityId, ItemId, LogisticsNodeId, LogisticsNodeSite, OptimizationResult,
    PowerRecipeId, RecipeId,
};
use end_opt::run_two_stage;
use generativity::make_guard;

use crate::dto::{
    BootstrapPayload, CatalogDto, CatalogFacilityDto, CatalogItemDto, ExternalSupplySlackDto,
    FacilityUsageDto, LogisticsEdgeDto, LogisticsGraphDto, LogisticsItemSummaryDto,
    LogisticsNodeDto, OutpostValueDto, PowerSummaryDto, SaleValueDto, SolvePayload, SummaryDto,
};
use crate::{Error, Lang, Result};

pub fn bootstrap(lang: Lang) -> Result<BootstrapPayload> {
    make_guard!(guard);
    let catalog = load_catalog(None, guard).map_err(Error::Catalog)?;

    let mut items = catalog
        .items()
        .iter()
        .map(|item| CatalogItemDto {
            key: item.key.as_str().into(),
            en: item.en.as_str().into(),
            zh: item.zh.as_str().into(),
        })
        .collect::<Vec<_>>();
    items.sort_by(|lhs, rhs| lhs.key.cmp(&rhs.key));

    let mut facilities = catalog
        .facilities()
        .iter()
        .map(|facility| CatalogFacilityDto {
            key: facility.key.as_str().into(),
            en: facility.en.as_str().into(),
            zh: facility.zh.as_str().into(),
        })
        .collect::<Vec<_>>();
    facilities.sort_by(|lhs, rhs| lhs.key.cmp(&rhs.key));

    // `lang` is currently not used by bootstrap payload fields, but keeping it in signature
    // makes the frontend contract symmetric with solve API and future localization extensions.
    let _ = lang;

    Ok(BootstrapPayload {
        catalog: CatalogDto {
            items: items.into_boxed_slice(),
            facilities: facilities.into_boxed_slice(),
        },
    })
}

pub fn solve_from_aic_toml(lang: Lang, aic_toml: &str) -> Result<SolvePayload> {
    make_guard!(catalog_guard);
    let catalog = load_catalog(None, catalog_guard).map_err(Error::Catalog)?;
    make_guard!(aic_guard);
    let aic = load_aic_from_str(aic_toml, &catalog, aic_guard).map_err(Error::Aic)?;

    make_guard!(result_guard);
    let solved = run_two_stage(&catalog, &aic, result_guard).map_err(Error::Optimize)?;
    // TODO remove this.
    let report_text = build_report_text(&catalog, &aic, &solved)?;

    Ok(SolvePayload {
        report_text,
        summary: build_summary(lang, &catalog, &aic, &solved)?,
        logistics_graph: build_logistics_graph(lang, &catalog, &aic, &solved)?,
    })
}

fn build_report_text<'cid, 'sid, 'rid>(
    _catalog: &Catalog<'cid>,
    _inputs: &AicInputs<'cid, 'sid>,
    _solved: &OptimizationResult<'cid, 'sid, 'rid>,
) -> Result<String> {
    // payload keeps the same shape, but report rendering is trimmed out.
    Ok(String::new())
}

fn build_summary<'cid, 'sid, 'rid>(
    lang: Lang,
    catalog: &Catalog<'cid>,
    inputs: &AicInputs<'cid, 'sid>,
    solved: &OptimizationResult<'cid, 'sid, 'rid>,
) -> Result<SummaryDto> {
    let stage1 = &solved.stage1;
    let stage2 = &solved.stage2;

    let outposts = stage2
        .outpost_values
        .iter()
        .map(|value| {
            let outpost = inputs.outpost(value.outpost_index);
            Ok::<_, Error>(OutpostValueDto {
                key: outpost.key.as_str().into(),
                name: outpost_name(lang, outpost).into(),
                value_per_min: value.value_per_min,
                cap_per_min: value.cap_per_min,
                ratio: value.ratio,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let mut top_sales = stage2
        .outpost_sales_qty
        .iter()
        .map(|sale| {
            let outpost = inputs.outpost(sale.outpost_index);

            let qty_per_min = sale.qty_per_min.get();
            let value_per_min = qty_per_min * sale.price as f64;

            Ok::<_, Error>(SaleValueDto {
                outpost_key: outpost.key.as_str().into(),
                outpost_name: outpost_name(lang, outpost).into(),
                item_key: item_key(catalog, sale.item)?.into(),
                item_name: item_name(lang, catalog, sale.item)?.into(),
                qty_per_min,
                value_per_min,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // 默认排序：按据点 key 倒序、再按物品 key。
    top_sales.sort_by(|lhs, rhs| {
        rhs.outpost_key
            .cmp(&lhs.outpost_key)
            .then_with(|| lhs.item_key.cmp(&rhs.item_key))
    });

    let facilities = stage2
        .machines_by_facility
        .iter()
        .map(|usage| {
            let facility_def = catalog.facility(usage.facility);
            Ok::<_, Error>(FacilityUsageDto {
                key: facility_key(catalog, usage.facility)?.into(),
                name: facility_name(lang, catalog, usage.facility)?.into(),
                machines: usage.machines,
                power_w: facility_def.power_w.get(),
                total_power_w: facility_def.power_w.get() * usage.machines,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let external_supply_slack = stage2
        .external_supply_slack
        .iter()
        .map(|row| {
            Ok::<_, Error>(ExternalSupplySlackDto {
                item_key: item_key(catalog, row.item)?.into(),
                item_name: item_name(lang, catalog, row.item)?.into(),
                supply_per_min: row.supply_per_min,
                slack_per_min: row.slack_per_min,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(SummaryDto {
        lang: lang.tag(),
        stage1_revenue_per_min: stage1.revenue_per_min,
        stage2_revenue_per_min: stage2.revenue_per_min,
        stage2_revenue_per_hour: stage2.revenue_per_min * 60.0,
        total_machines: stage2.total_machines,
        total_thermal_banks: stage2.total_thermal_banks,
        power: stage2.power.as_ref().map(|power| PowerSummaryDto {
            external_production_w: power.external_production_w,
            external_consumption_w: power.external_consumption_w,
            thermal_generation_w: power.thermal_generation_w,
            machine_consumption_w: power.machine_consumption_w,
            total_gen_w: power.total_gen_w,
            total_use_w: power.total_use_w,
            margin_w: power.margin_w,
        }),
        outposts: outposts.into_boxed_slice(),
        top_sales: top_sales.into_boxed_slice(),
        facilities: facilities.into_boxed_slice(),
        external_supply_slack: external_supply_slack.into_boxed_slice(),
    })
}

fn build_logistics_graph<'cid, 'sid, 'rid>(
    lang: Lang,
    catalog: &Catalog<'cid>,
    inputs: &AicInputs<'cid, 'sid>,
    solved: &OptimizationResult<'cid, 'sid, 'rid>,
) -> Result<LogisticsGraphDto> {
    // 构建配方机器数查找表
    let recipe_machines: BTreeMap<RecipeId<'cid>, u32> = solved
        .stage2
        .recipes_used
        .iter()
        .map(|r| (r.recipe_index, r.machines.get()))
        .collect();
    // 构建热能池数量查找表
    let thermal_banks: BTreeMap<PowerRecipeId<'cid>, u32> = solved
        .stage2
        .thermal_banks_used
        .iter()
        .map(|t| (t.power_recipe_index, t.banks.get()))
        .collect();

    let node_by_id = solved
        .logistics
        .nodes
        .iter()
        .map(|node| (node.id, node))
        .collect::<BTreeMap<_, _>>();

    let mut nodes = solved
        .logistics
        .nodes
        .iter()
        .map(|node| {
            let (kind, label) = describe_logistics_site(
                lang,
                catalog,
                inputs,
                &node.site,
                &recipe_machines,
                &thermal_banks,
            )?;
            Ok::<_, Error>(LogisticsNodeDto {
                id: logistics_node_id(node.id),
                kind,
                label,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    nodes.sort_by(|lhs, rhs| lhs.id.cmp(&rhs.id));

    let mut edges = Vec::<LogisticsEdgeDto>::new();
    let mut item_summary = BTreeMap::<ItemId<'cid>, ItemGraphStats>::new();

    for edge in &solved.logistics.edges {
        let item = edge.item;
        let item_key: Box<str> = item_key(catalog, item)?.into();
        let item_name: Box<str> = item_name(lang, catalog, item)?.into();

        if !node_by_id.contains_key(&edge.from) {
            return Err(Error::MissingLogisticsNode {
                item: item.as_u32(),
                node: edge.from.as_u32(),
            });
        }
        if !node_by_id.contains_key(&edge.to) {
            return Err(Error::MissingLogisticsNode {
                item: item.as_u32(),
                node: edge.to.as_u32(),
            });
        }

        edges.push(LogisticsEdgeDto {
            id: format!("{}:{}:{}", item_key, edge.from.as_u32(), edge.to.as_u32())
                .into_boxed_str(),
            item_key,
            item_name,
            source: logistics_node_id(edge.from),
            target: logistics_node_id(edge.to),
            flow_per_min: edge.flow_per_min.get(),
        });

        let entry = item_summary.entry(item).or_default();
        entry.edge_count += 1;
        entry.node_ids.insert(edge.from.as_u32());
        entry.node_ids.insert(edge.to.as_u32());
        entry.total_flow_per_min += edge.flow_per_min.get();
    }

    let mut items = item_summary
        .into_iter()
        .map(|(item, stats)| {
            Ok::<_, Error>(LogisticsItemSummaryDto {
                item_key: item_key(catalog, item)?.into(),
                item_name: item_name(lang, catalog, item)?.into(),
                edge_count: stats.edge_count,
                node_count: stats.node_ids.len(),
                total_flow_per_min: stats.total_flow_per_min,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    items.sort_by(|lhs, rhs| lhs.item_key.cmp(&rhs.item_key));

    Ok(LogisticsGraphDto {
        items: items.into_boxed_slice(),
        nodes: nodes.into_boxed_slice(),
        edges: edges.into_boxed_slice(),
    })
}

#[derive(Debug, Default)]
struct ItemGraphStats {
    edge_count: usize,
    node_ids: BTreeSet<u32>,
    total_flow_per_min: f64,
}

fn describe_logistics_site<'cid, 'sid>(
    lang: Lang,
    catalog: &Catalog<'cid>,
    inputs: &AicInputs<'cid, 'sid>,
    site: &LogisticsNodeSite<'cid, 'sid>,
    recipe_machines: &BTreeMap<RecipeId<'cid>, u32>,
    thermal_banks: &BTreeMap<PowerRecipeId<'cid>, u32>,
) -> Result<(Box<str>, Box<str>)> {
    match site {
        LogisticsNodeSite::ExternalSupply { item } => Ok((
            "external_supply".into(),
            match lang {
                Lang::Zh => {
                    format!("外部供给 ({})", item_name(lang, catalog, *item)?).into_boxed_str()
                }
                Lang::En => format!("External supply ({})", item_name(lang, catalog, *item)?)
                    .into_boxed_str(),
            },
        )),
        LogisticsNodeSite::ExternalConsumption { item } => Ok((
            "external_consumption".into(),
            match lang {
                Lang::Zh => {
                    format!("外部消耗 ({})", item_name(lang, catalog, *item)?).into_boxed_str()
                }
                Lang::En => format!(
                    "External consumption ({})",
                    item_name(lang, catalog, *item)?
                )
                .into_boxed_str(),
            },
        )),
        LogisticsNodeSite::RecipeGroup { recipe_index } => {
            let recipe = catalog.recipe(*recipe_index);
            let facility = facility_name(lang, catalog, recipe.facility)?;
            let machines = recipe_machines.get(recipe_index).copied().unwrap_or(1);
            let product_names: Vec<String> = recipe
                .products
                .iter()
                .map(|p| item_name(lang, catalog, p.item).map(String::from))
                .collect::<Result<Vec<_>>>()?;
            let products_str = product_names.join(", ");
            Ok((
                "recipe_group".into(),
                match lang {
                    Lang::Zh => {
                        format!("{} x{} → {}", facility, machines, products_str).into_boxed_str()
                    }
                    Lang::En => {
                        format!("{} x{} → {}", facility, machines, products_str).into_boxed_str()
                    }
                },
            ))
        }
        LogisticsNodeSite::OutpostSale {
            outpost_index,
            item,
        } => {
            let outpost = inputs.outpost(*outpost_index);
            Ok((
                "outpost_sale".into(),
                match lang {
                    Lang::Zh => format!(
                        "{} 出售 ({})",
                        outpost_name(lang, outpost),
                        item_name(lang, catalog, *item)?
                    )
                    .into_boxed_str(),
                    Lang::En => format!(
                        "{} sale ({})",
                        outpost_name(lang, outpost),
                        item_name(lang, catalog, *item)?
                    )
                    .into_boxed_str(),
                },
            ))
        }
        LogisticsNodeSite::ThermalBankGroup {
            power_recipe_index,
            item: _,
        } => {
            let banks = thermal_banks.get(power_recipe_index).copied().unwrap_or(1);
            Ok((
                "thermal_bank_group".into(),
                match lang {
                    Lang::Zh => format!("热能池 x{}", banks).into_boxed_str(),
                    Lang::En => format!("Thermal bank x{}", banks).into_boxed_str(),
                },
            ))
        }
        LogisticsNodeSite::WarehouseStockpile { item } => Ok((
            "warehouse_stockpile".into(),
            match lang {
                Lang::Zh => format!("仓库 ({})", item_name(lang, catalog, *item)?).into_boxed_str(),
                Lang::En => format!(
                    "Stockpile to warehouse ({})",
                    item_name(lang, catalog, *item)?
                )
                .into_boxed_str(),
            },
        )),
    }
}

fn outpost_name<'a, 'id>(lang: Lang, outpost: &'a end_model::OutpostInput<'id>) -> &'a str {
    match lang {
        Lang::Zh => outpost.zh.as_deref().unwrap_or(outpost.key.as_str()),
        Lang::En => outpost.en.as_deref().unwrap_or(outpost.key.as_str()),
    }
}

fn item_key<'a, 'id>(catalog: &'a Catalog<'id>, item: ItemId<'id>) -> Result<&'a str> {
    Ok(catalog.item(item).key.as_str())
}

fn item_name<'a, 'id>(lang: Lang, catalog: &'a Catalog<'id>, item: ItemId<'id>) -> Result<&'a str> {
    let item = catalog.item(item);
    Ok(match lang {
        Lang::Zh => item.zh.as_str(),
        Lang::En => item.en.as_str(),
    })
}

fn facility_key<'a, 'id>(catalog: &'a Catalog<'id>, facility: FacilityId<'id>) -> Result<&'a str> {
    Ok(catalog.facility(facility).key.as_str())
}

fn facility_name<'a, 'id>(
    lang: Lang,
    catalog: &'a Catalog<'id>,
    facility: FacilityId<'id>,
) -> Result<&'a str> {
    let facility = catalog.facility(facility);
    Ok(match lang {
        Lang::Zh => facility.zh.as_str(),
        Lang::En => facility.en.as_str(),
    })
}

fn logistics_node_id(node: LogisticsNodeId<'_>) -> Box<str> {
    format!("n{}", node.as_u32()).into_boxed_str()
}
