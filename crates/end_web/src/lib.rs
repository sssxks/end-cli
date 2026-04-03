mod api;
mod dto;
mod error;
mod ffi;
mod lang;

pub use api::{bootstrap, solve_from_aic_toml};
pub use dto::{
    BootstrapPayload, CatalogDto, CatalogFacilityDto, CatalogItemDto, ExternalSupplySlackDto,
    FacilityUsageDto, LogisticsEdgeDto, LogisticsGraphDto, LogisticsItemSummaryDto,
    LogisticsNodeDto, OutpostValueDto, PowerSummaryDto, SaleValueDto, SolvePayload, SummaryDto,
};
pub use error::{Error, Result};
pub use ffi::{end_web_bootstrap, end_web_free_slice, end_web_solve_from_aic_toml};
pub use lang::Lang;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)]

    use super::{Lang, bootstrap, solve_from_aic_toml};
    use end_io::{default_aic_toml, load_catalog};
    use generativity::make_guard;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn bootstrap_returns_catalog_items() {
        let payload = bootstrap(Lang::Zh).expect("bootstrap should succeed");
        assert!(
            !payload.catalog.items.is_empty(),
            "catalog items should not be empty"
        );
        assert!(
            !payload.catalog.facilities.is_empty(),
            "catalog facilities should not be empty"
        );
    }

    #[test]
    fn solve_runs_with_builtin_default_aic() {
        make_guard!(guard);
        let catalog = load_catalog(None, guard).expect("builtin catalog should load");
        let aic_toml = default_aic_toml(&catalog).expect("default aic should build");

        let payload = solve_from_aic_toml(Lang::Zh, &aic_toml).expect("solve should succeed");
        assert!(
            payload.summary.stage2_revenue_per_min >= 0.0,
            "revenue should be non-negative"
        );
        assert!(
            !payload.logistics_graph.nodes.is_empty(),
            "default scenario should include logistics nodes"
        );
        assert!(
            payload
                .logistics_graph
                .nodes
                .iter()
                .any(|node| node.kind.as_ref() == "external_consumption"),
            "default scenario should include external consumption nodes"
        );
    }

    #[test]
    fn solve_rejects_invalid_toml() {
        let err = solve_from_aic_toml(
            Lang::Zh,
            "version = 2\n[power]\nexternal_consumption = 'oops'",
        )
        .expect_err("invalid toml should fail");
        let msg = err.to_string();
        assert!(
            msg.contains("Aic parse failed"),
            "error message should mention aic parse"
        );
    }

    #[test]
    fn solve_creates_per_item_warehouse_stockpile_nodes() {
        make_guard!(guard);
        let catalog = load_catalog(None, guard).expect("builtin catalog should load");
        assert!(
            catalog.items().len() >= 2,
            "builtin catalog should include at least two items"
        );

        let item_a = catalog.items()[0].key.as_str();
        let item_b = catalog.items()[1].key.as_str();
        let aic_toml = format!(
            r#"
version = 2

[supply_per_min]
"{item_a}" = 5
"{item_b}" = 7
"#
        );

        let payload = solve_from_aic_toml(Lang::En, &aic_toml).expect("solve should succeed");
        let warehouse_node_ids = payload
            .logistics_graph
            .nodes
            .iter()
            .filter(|node| node.kind.as_ref() == "warehouse_stockpile")
            .map(|node| node.id.as_ref())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            warehouse_node_ids.len(),
            2,
            "stockpiled items should not share one warehouse node"
        );

        let stockpile_target_by_item = payload
            .logistics_graph
            .edges
            .iter()
            .filter(|edge| warehouse_node_ids.contains(edge.target.as_ref()))
            .fold(BTreeMap::<&str, &str>::new(), |mut acc, edge| {
                acc.insert(edge.item_key.as_ref(), edge.target.as_ref());
                acc
            });
        assert_eq!(
            stockpile_target_by_item.len(),
            2,
            "each supplied item should have stockpile flow"
        );
        assert_ne!(
            stockpile_target_by_item.get(item_a),
            stockpile_target_by_item.get(item_b),
            "different items should end at different warehouse nodes"
        );
    }
}
