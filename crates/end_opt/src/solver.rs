use crate::consts::LOGISTICS_EPS;
use crate::error::{Error, Result};
use crate::logistics::build_logistics_plan;
use end_model::{
    AicInputs, Catalog, ExternalSupplySlack, FacilityId, FacilityMachineCount, ItemId,
    ItemStockpile, ItemVec, OptimizationResult, OutpostId, OutpostSaleQty, OutpostValue, PosF64,
    PowerConfig, PowerRecipeId, PowerSummary, RecipeId, RecipeUsage, Stage2Weights, StageSolution,
    ThermalBankUsage,
};
use generativity::Guard;
use good_lp::{
    Expression, Solution, SolverModel, Variable, constraint, default_solver, variable, variables,
};
use smallvec::SmallVec;
use std::collections::{BTreeSet, HashMap};
use std::num::NonZeroU32;

/// Maximum tolerated distance from an integer value for decoded integer variables.
pub const NEAR_INT_EPS: f64 = 1e-6;

#[derive(Debug, Clone, Copy)]
enum StageObjective {
    MaxRevenue,
    MinMachines {
        revenue_floor_per_min: f64,
    },
    MaxPowerSlack {
        revenue_floor_per_min: f64,
    },
    MaxMoneySlack {
        revenue_floor_per_min: f64,
    },
    Weighted {
        revenue_floor_per_min: f64,
        weights: Stage2Weights,
        power_slack_scale: f64,
        money_slack_scale: f64,
    },
}

#[derive(Debug, Clone, Copy)]
struct Stage2SlackScale {
    power_slack_max_w: f64,
    money_slack_max_per_min: f64,
}

#[derive(Debug, Clone, Copy)]
struct PowerModel {
    enabled: bool,
    external_production_w: u32,
    external_consumption_w: u32,
}

#[derive(Debug, Clone)]
struct OutpostVars<'cid, 'sid> {
    outpost_index: OutpostId<'sid>,
    money_cap_per_hour: u32,
    sell_lines: Vec<SellLineVars<'cid>>,
}

#[derive(Debug, Clone)]
struct SellLineVars<'cid> {
    item: ItemId<'cid>,
    price: u32,
    qty_real: Variable,
    qty_virtual: Option<Variable>,
}

#[derive(Debug, Clone)]
struct RecipeVars<'id> {
    recipe_index: RecipeId<'id>,
    facility: FacilityId<'id>,
    facility_power_w: u32,
    x: Variable,
    y: Variable,
    throughput_per_min: f64,
    net: SmallVec<[(ItemId<'id>, f64); 4]>,
}

#[derive(Debug, Clone)]
struct PowerVars<'id> {
    power_recipe_index: PowerRecipeId<'id>,
    ingredient: ItemId<'id>,
    power_w: u32,
    duration_s: u32,
    z: Variable,
}

/// Run the two-stage optimizer:
/// 1) maximize revenue, 2) optimize configurable stage2 objective under near-optimal revenue floor.
pub fn run_two_stage<'cid, 'sid, 'rid>(
    catalog: &Catalog<'cid>,
    aic: &AicInputs<'cid, 'sid>,
    guard: Guard<'rid>,
) -> Result<OptimizationResult<'cid, 'sid, 'rid>> {
    let stage1 = solve_stage(catalog, aic, StageObjective::MaxRevenue)?;

    let revenue_floor_per_min = stage1.revenue_per_min;
    let weights: Stage2Weights = aic.stage2_weights();
    let stage2 = match weights.active_target_count() {
        0 => stage1.clone(),
        1 if weights.min_machines > 0.0 => solve_stage(
            catalog,
            aic,
            StageObjective::MinMachines {
                revenue_floor_per_min,
            },
        )?,
        1 if weights.max_power_slack > 0.0 => solve_stage(
            catalog,
            aic,
            StageObjective::MaxPowerSlack {
                revenue_floor_per_min,
            },
        )?,
        1 => solve_stage(
            catalog,
            aic,
            StageObjective::MaxMoneySlack {
                revenue_floor_per_min,
            },
        )?,
        _ => {
            let Stage2SlackScale {
                power_slack_max_w,
                money_slack_max_per_min,
            } = solve_stage2_slack_scale(catalog, aic, revenue_floor_per_min, weights)?;
            solve_stage(
                catalog,
                aic,
                StageObjective::Weighted {
                    revenue_floor_per_min,
                    weights,
                    power_slack_scale: power_slack_max_w,
                    money_slack_scale: money_slack_max_per_min,
                },
            )?
        }
    };
    let logistics = build_logistics_plan(catalog, aic, &stage2, guard)?;

    Ok(OptimizationResult {
        stage1,
        stage2,
        logistics,
    })
}

fn solve_stage2_slack_scale<'cid, 'sid>(
    catalog: &Catalog<'cid>,
    aic: &AicInputs<'cid, 'sid>,
    revenue_floor_per_min: f64,
    weights: Stage2Weights,
) -> Result<Stage2SlackScale> {
    let power_slack_max_w = if weights.max_power_slack > 0.0 {
        let power = solve_stage(
            catalog,
            aic,
            StageObjective::MaxPowerSlack {
                revenue_floor_per_min,
            },
        )?;
        power
            .power
            .as_ref()
            .map(|summary| summary.margin_w as f64)
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let money_slack_max_per_min = if weights.max_money_slack > 0.0 {
        let money = solve_stage(
            catalog,
            aic,
            StageObjective::MaxMoneySlack {
                revenue_floor_per_min,
            },
        )?;
        money.money_slack_per_min
    } else {
        0.0
    };

    Ok(Stage2SlackScale {
        power_slack_max_w,
        money_slack_max_per_min,
    })
}

fn solve_stage<'cid, 'sid>(
    catalog: &Catalog<'cid>,
    aic: &AicInputs<'cid, 'sid>,
    objective: StageObjective,
) -> Result<StageSolution<'cid, 'sid>> {
    let power_model = match aic.power_config() {
        PowerConfig::Disabled => PowerModel {
            enabled: false,
            external_production_w: 0,
            external_consumption_w: 0,
        },
        PowerConfig::Enabled {
            external_production_w,
            external_consumption_w,
        } => PowerModel {
            enabled: true,
            external_production_w,
            external_consumption_w,
        },
    };

    if matches!(objective, StageObjective::MaxPowerSlack { .. }) && !power_model.enabled {
        return Err(Error::InvalidInput {
            message: "cannot optimize max_power_slack when power.enabled = false"
                .to_string()
                .into_boxed_str(),
        });
    }

    if let StageObjective::Weighted { weights, .. } = objective
        && weights.max_power_slack > 0.0
        && !power_model.enabled
    {
        return Err(Error::InvalidInput {
            message: "objective.max_power_slack must be 0 when power.enabled = false"
                .to_string()
                .into_boxed_str(),
        });
    }

    let enable_virtual_sales = matches!(objective, StageObjective::MaxMoneySlack { .. })
        || matches!(
            objective,
            StageObjective::Weighted { weights, .. } if weights.max_money_slack > 0.0
        );
    let enable_power_slack = power_model.enabled
        && (matches!(objective, StageObjective::MaxPowerSlack { .. })
            || matches!(
                objective,
                StageObjective::Weighted { weights, .. } if weights.max_power_slack > 0.0
            ));

    let mut vars = variables!();

    let recipe_vars = catalog
        .recipes_with_id()
        .filter(|(_, recipe)| catalog.facility_available_in_region(recipe.facility, aic.region()))
        .map(|(recipe_index, recipe)| {
            let facility = catalog.facility(recipe.facility);
            let x = vars.add(variable().min(0.0));
            let y = vars.add(variable().integer().min(0.0));
            let time_s = recipe.time_s as f64;

            let net = recipe
                .ingredients
                .iter()
                .map(|stack| (stack.item, -(stack.count.get() as f64)))
                .chain(
                    recipe
                        .products
                        .iter()
                        .map(|stack| (stack.item, stack.count.get() as f64)),
                )
                .fold(
                    SmallVec::with_capacity(recipe.ingredients.len() + recipe.products.len()),
                    |mut net, (item, delta)| {
                        add_recipe_net_delta(&mut net, item, delta);
                        net
                    },
                );

            RecipeVars {
                recipe_index,
                facility: recipe.facility,
                facility_power_w: facility.power_w.get(),
                x,
                y,
                throughput_per_min: 60.0 / time_s,
                net,
            }
        })
        .collect::<Vec<_>>();

    let power_vars = if power_model.enabled {
        catalog
            .power_recipes_with_id()
            .map(|(id, p)| PowerVars {
                power_recipe_index: id,
                ingredient: p.ingredient.item,
                power_w: p.power_w.get(),
                duration_s: p.time_s.get(),
                z: vars.add(variable().integer().min(0.0)),
            })
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let outpost_vars = aic
        .outposts_with_id()
        .map(|(id, outpost)| {
            let sell_lines = outpost
                .prices
                .iter()
                .map(|(item, price)| {
                    let qty_real = vars.add(variable().min(0.0));
                    let qty_virtual = enable_virtual_sales
                        .then_some(catalog.item(item))
                        .filter(|item_def| !item_def.is_fluid)
                        .map(|_| vars.add(variable().min(0.0)));
                    SellLineVars {
                        item,
                        price,
                        qty_real,
                        qty_virtual,
                    }
                })
                .collect::<Vec<_>>();

            OutpostVars {
                outpost_index: id,
                money_cap_per_hour: outpost.money_cap_per_hour,
                sell_lines,
            }
        })
        .collect::<Vec<_>>();

    let revenue: Expression = outpost_vars
        .iter()
        .flat_map(|ov| ov.sell_lines.iter())
        .map(|line| line.qty_real * line.price)
        .sum();

    let money_slack: Expression = outpost_vars
        .iter()
        .flat_map(|ov| ov.sell_lines.iter())
        .filter_map(|line| line.qty_virtual.map(|qty| qty * line.price))
        .sum();

    let power_slack_var = enable_power_slack.then(|| vars.add(variable().min(0.0)));

    let total_machines: Expression = recipe_vars.iter().map(|rv| rv.y).sum();
    let total_thermal_banks: Expression = power_vars.iter().map(|pv| pv.z).sum();
    let machine_power_use = recipe_vars
        .iter()
        .map(|rv| rv.y * rv.facility_power_w)
        .sum::<Expression>();
    let thermal_generation = power_vars
        .iter()
        .map(|pv| pv.z * pv.power_w)
        .sum::<Expression>();

    let power_gen = if power_model.enabled {
        Expression::from(power_model.external_production_w as f64) + thermal_generation.clone()
    } else {
        Expression::from(0.0)
    };

    let power_use = if power_model.enabled {
        Expression::from(power_model.external_consumption_w as f64) + machine_power_use.clone()
    } else {
        Expression::from(0.0)
    };

    // Build per-item balances by dispatching each contribution to its item bucket.
    // Seed from external supplies first.
    let item_balance = aic.supply_per_min().iter().fold(
        ItemVec::filled(catalog, Expression::from(0.0)),
        |mut item_balance, (item, supply)| {
            item_balance[item] = Expression::from(supply.get());
            item_balance
        },
    );

    let item_balance = aic.external_consumption_per_min().iter().fold(
        item_balance,
        |mut item_balance, (item, consume)| {
            item_balance[item] -= consume.get();
            item_balance
        },
    );

    let item_balance = recipe_vars
        .iter()
        .fold(item_balance, |mut item_balance, rv| {
            rv.net.iter().for_each(|(item, delta)| {
                item_balance[*item] += *delta * rv.x;
            });
            item_balance
        });

    let (item_balance, virtual_sales_by_item) = outpost_vars.iter().fold(
        (
            item_balance,
            ItemVec::filled(catalog, Expression::from(0.0)),
        ),
        |(mut item_balance, mut virtual_sales_by_item), ov| {
            ov.sell_lines.iter().for_each(|line| {
                item_balance[line.item] -= line.qty_real;
                if let Some(qty_virtual) = line.qty_virtual {
                    item_balance[line.item] -= qty_virtual;
                    virtual_sales_by_item[line.item] += qty_virtual;
                }
            });
            (item_balance, virtual_sales_by_item)
        },
    );

    let item_balance = power_vars
        .iter()
        .fold(item_balance, |mut item_balance, pv| {
            let consume_per_min = 60.0 / pv.duration_s as f64;
            item_balance[pv.ingredient] -= consume_per_min * pv.z;
            item_balance
        });

    let machine_count: Expression = total_machines.clone() + total_thermal_banks.clone();
    let power_slack_expr = power_slack_var
        .map(Expression::from)
        .unwrap_or_else(|| power_gen.clone() - power_use.clone());

    let mut model = match objective {
        StageObjective::MaxRevenue => vars.maximise(revenue.clone()).using(default_solver),
        StageObjective::MinMachines {
            revenue_floor_per_min,
        } => vars
            .minimise(machine_count.clone())
            .using(default_solver)
            .with(constraint!(revenue.clone() >= revenue_floor_per_min)),
        StageObjective::MaxPowerSlack {
            revenue_floor_per_min,
        } => vars
            .maximise(power_slack_expr.clone())
            .using(default_solver)
            .with(constraint!(revenue.clone() >= revenue_floor_per_min)),
        StageObjective::MaxMoneySlack {
            revenue_floor_per_min,
        } => vars
            .maximise(money_slack.clone())
            .using(default_solver)
            .with(constraint!(revenue.clone() >= revenue_floor_per_min)),
        StageObjective::Weighted {
            revenue_floor_per_min,
            weights,
            power_slack_scale,
            money_slack_scale,
        } => {
            let power_denom = power_slack_scale.max(1e-9);
            let money_denom = money_slack_scale.max(1e-9);
            let weighted_obj = machine_count.clone() * weights.min_machines
                - power_slack_expr.clone() * (weights.max_power_slack / power_denom)
                - money_slack.clone() * (weights.max_money_slack / money_denom);
            vars.minimise(weighted_obj)
                .using(default_solver)
                .with(constraint!(revenue.clone() >= revenue_floor_per_min))
        }
    };

    model = outpost_vars.iter().fold(model, |model, ov| {
        let outpost_value: Expression = ov
            .sell_lines
            .iter()
            .map(|line| line.qty_real * line.price)
            .sum();
        model.with(constraint!(
            outpost_value <= ov.money_cap_per_hour as f64 / 60.0
        ))
    });

    if power_model.enabled {
        model = model.with(constraint!(&power_gen >= power_use.clone()));
        if let Some(power_slack) = power_slack_var {
            model = model.with(constraint!(
                power_slack <= power_gen.clone() - power_use.clone()
            ));
        }
    }

    model = recipe_vars.iter().fold(model, |model, rv| {
        model.with(constraint!(rv.x <= rv.throughput_per_min * rv.y))
    });

    model = aic
        .facility_machines_max()
        .iter()
        .fold(model, |model, (facility, max_machines)| {
            let facility_machines: Expression = recipe_vars
                .iter()
                .filter(|rv| rv.facility == facility)
                .map(|rv| rv.y)
                .sum();
            model.with(constraint!(facility_machines <= max_machines as f64))
        });

    // Apply item balance constraints:
    // - For fluids: equality (must be exactly 0, no storage allowed)
    // - For non-fluids: inequality (>= 0, surplus can go to warehouse)
    model = catalog
        .items_with_id()
        .fold(model, |model, (item_id, item_def)| {
            let expr = &item_balance[item_id];
            if item_def.is_fluid {
                model.with(constraint!(expr.clone() == 0.0))
            } else {
                model.with(constraint!(expr.clone() >= 0.0))
            }
        });

    let solution = model.solve().map_err(|source| Error::Solver { source })?;

    let revenue_per_min = solution.eval(&revenue);
    let total_machines = near_u32(|| "total_machines".into(), solution.eval(&total_machines))?;
    let total_thermal_banks = near_u32(
        || "total_thermal_banks".into(),
        solution.eval(&total_thermal_banks),
    )?;
    let power = if power_model.enabled {
        let thermal_generation_w = near_u32(
            || "thermal_generation_w".into(),
            solution.eval(&thermal_generation),
        )?;
        let machine_consumption_w = near_u32(
            || "machine_consumption_w".into(),
            solution.eval(&machine_power_use),
        )?;
        let total_gen_w = near_u32(|| "power.total_gen_w".into(), solution.eval(&power_gen))?;
        let total_use_w = near_u32(|| "power.total_use_w".into(), solution.eval(&power_use))?;
        let margin_w = total_gen_w
            .checked_sub(total_use_w)
            .ok_or(Error::InvalidInput {
                message: format!(
                    "decoded power total_gen_w {} < total_use_w {}",
                    total_gen_w, total_use_w
                )
                .into_boxed_str(),
            })?;
        Some(PowerSummary {
            external_production_w: power_model.external_production_w,
            external_consumption_w: power_model.external_consumption_w,
            thermal_generation_w,
            machine_consumption_w,
            total_gen_w,
            total_use_w,
            margin_w,
        })
    } else {
        None
    };
    let money_slack_per_min = solution.eval(&money_slack);

    let mut outpost_sales_qty = Vec::with_capacity(
        outpost_vars
            .iter()
            .map(|ov| ov.sell_lines.len())
            .sum::<usize>(),
    );
    let outpost_values = outpost_vars
        .iter()
        .map(|ov| -> Result<_> {
            let mut value_per_min = 0.0;
            for line in &ov.sell_lines {
                let qty_value = solution.value(line.qty_real);
                if qty_value <= LOGISTICS_EPS {
                    continue;
                }
                let qty_per_min = PosF64::new(qty_value).ok_or(Error::InvalidPositiveFlow {
                    context: format!(
                        "outpost_sales_qty[outpost={},item={}]",
                        ov.outpost_index.as_u32(),
                        line.item.as_u32()
                    )
                    .into_boxed_str(),
                    value: qty_value,
                })?;
                let value = line.price as f64 * qty_value;
                outpost_sales_qty.push(OutpostSaleQty {
                    outpost_index: ov.outpost_index,
                    item: line.item,
                    qty_per_min,
                    price: line.price,
                });
                value_per_min += value;
            }

            let cap_per_min = ov.money_cap_per_hour as f64 / 60.0;
            let ratio = value_per_min / cap_per_min;

            Ok(OutpostValue {
                outpost_index: ov.outpost_index,
                value_per_min,
                cap_per_min,
                ratio,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let (machines_by_facility_map, mut recipes_used) = recipe_vars.iter().try_fold(
        (
            HashMap::<FacilityId<'cid>, u32>::new(),
            Vec::with_capacity(recipe_vars.len()),
        ),
        |(mut machines_by_facility_map, mut recipes_used), rv| -> Result<_> {
            let machines = near_u32(
                || format!("recipes[{}].machines", rv.recipe_index.as_u32()).into_boxed_str(),
                solution.value(rv.y),
            )?;
            let executions_per_min = solution.value(rv.x);
            if let Some(machines) = NonZeroU32::new(machines) {
                *machines_by_facility_map.entry(rv.facility).or_insert(0) += machines.get();
                recipes_used.push(RecipeUsage {
                    recipe_index: rv.recipe_index,
                    machines,
                    executions_per_min,
                });
            }
            Ok((machines_by_facility_map, recipes_used))
        },
    )?;

    let mut machines_by_facility = machines_by_facility_map
        .into_iter()
        .map(|(facility, machines)| FacilityMachineCount { facility, machines })
        .collect::<Vec<_>>();
    machines_by_facility.sort_by(|a, b| b.machines.cmp(&a.machines));

    recipes_used.sort_by(|a, b| b.machines.cmp(&a.machines));

    let mut thermal_banks_used = Vec::with_capacity(power_vars.len());
    for pv in &power_vars {
        let banks = near_u32(
            || format!("power_recipes[{}].banks", pv.power_recipe_index.as_u32()).into_boxed_str(),
            solution.value(pv.z),
        )?;
        if banks == 0 {
            continue;
        }

        let banks = NonZeroU32::new(banks).ok_or_else(|| Error::InvalidInput {
            message: format!(
                "power_recipes[{}].banks decoded as zero unexpectedly",
                pv.power_recipe_index.as_u32()
            )
            .into_boxed_str(),
        })?;
        thermal_banks_used.push(ThermalBankUsage {
            power_recipe_index: pv.power_recipe_index,
            ingredient: pv.ingredient,
            banks,
            power_w: pv.power_w,
            duration_s: pv.duration_s,
        });
    }
    thermal_banks_used.sort_by(|a, b| b.banks.cmp(&a.banks));

    let mut external_supply_slack = aic
        .supply_per_min()
        .iter()
        .map(|(item, supply)| {
            let expr = &item_balance[item];
            ExternalSupplySlack {
                item,
                slack_per_min: solution.eval(expr),
                supply_per_min: supply.get(),
            }
        })
        .collect::<Vec<_>>();
    external_supply_slack.sort_by(|a, b| a.slack_per_min.total_cmp(&b.slack_per_min));

    // Per-item stockpile quantity in units/min for warehouse-storable items.
    //
    // Motivation: stage2 may introduce virtual sales variables to maximize potential money value.
    // Those virtual quantities are not physically sold in game and should be interpreted as
    // items ending up in warehouse, so for non-fluids we define:
    //   stockpile(item) = item_balance_slack(item) + virtual_sales_qty(item).
    let mut touched_items = BTreeSet::<ItemId<'cid>>::new();
    for (item, _) in aic.supply_per_min().iter() {
        touched_items.insert(item);
    }
    for (item, _) in aic.external_consumption_per_min().iter() {
        touched_items.insert(item);
    }
    for rv in &recipe_vars {
        for (item, _) in &rv.net {
            touched_items.insert(*item);
        }
    }
    for ov in &outpost_vars {
        for line in &ov.sell_lines {
            touched_items.insert(line.item);
        }
    }
    for pv in &power_vars {
        touched_items.insert(pv.ingredient);
    }

    let mut item_stockpile = Vec::<ItemStockpile<'cid>>::new();
    for item in touched_items {
        if catalog.item(item).is_fluid {
            continue;
        }
        let slack = solution.eval(&item_balance[item]);
        let virtual_qty = solution.eval(&virtual_sales_by_item[item]);
        let stockpile = slack + virtual_qty;
        if stockpile > LOGISTICS_EPS {
            item_stockpile.push(ItemStockpile {
                item,
                qty_per_min: stockpile,
            });
        }
    }
    item_stockpile.sort_by(|a, b| a.item.as_u32().cmp(&b.item.as_u32()));

    Ok(StageSolution {
        revenue_per_min,
        total_machines,
        total_thermal_banks,
        power,
        money_slack_per_min,
        outpost_values: outpost_values.into_boxed_slice(),
        outpost_sales_qty: outpost_sales_qty.into_boxed_slice(),
        machines_by_facility: machines_by_facility.into_boxed_slice(),
        recipes_used: recipes_used.into_boxed_slice(),
        thermal_banks_used: thermal_banks_used.into_boxed_slice(),
        external_supply_slack: external_supply_slack.into_boxed_slice(),
        item_stockpile: item_stockpile.into_boxed_slice(),
    })
}

fn near_u32<F>(var_name: F, value: f64) -> Result<u32>
where
    F: FnOnce() -> Box<str>,
{
    if !value.is_finite() {
        return Err(Error::OutOfRange {
            var_name: var_name(),
            value,
        });
    }

    let nearest = value.round();
    let delta = (value - nearest).abs();
    if delta > NEAR_INT_EPS {
        return Err(Error::NotNearInt {
            var_name: var_name(),
            value,
            nearest,
            delta,
            eps: NEAR_INT_EPS,
        });
    }

    if nearest < 0.0 || nearest > u32::MAX as f64 {
        return Err(Error::OutOfRange {
            var_name: var_name(),
            value,
        });
    }

    Ok(nearest as u32)
}

fn add_recipe_net_delta<'id>(
    net: &mut SmallVec<[(ItemId<'id>, f64); 4]>,
    item: ItemId<'id>,
    delta: f64,
) {
    if let Some((_, acc)) = net.iter_mut().find(|(net_item, _)| *net_item == item) {
        *acc += delta;
        return;
    }
    net.push((item, delta));
}
