use serde::Serialize;
use std::error::Error as _;

use crate::Result;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub catalog: CatalogDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogDto {
    pub items: Box<[CatalogItemDto]>,
    pub facilities: Box<[CatalogFacilityDto]>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogItemDto {
    pub key: Box<str>,
    pub en: Box<str>,
    pub zh: Box<str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogFacilityDto {
    pub key: Box<str>,
    pub en: Box<str>,
    pub zh: Box<str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SolvePayload {
    pub report_text: String,
    pub summary: SummaryDto,
    pub logistics_graph: LogisticsGraphDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryDto {
    pub lang: &'static str,
    pub stage1_revenue_per_min: f64,
    pub stage2_revenue_per_min: f64,
    pub stage2_revenue_per_hour: f64,
    pub total_machines: u32,
    pub total_thermal_banks: u32,
    pub power: Option<PowerSummaryDto>,
    pub outposts: Box<[OutpostValueDto]>,
    pub top_sales: Box<[SaleValueDto]>,
    pub facilities: Box<[FacilityUsageDto]>,
    pub external_supply_slack: Box<[ExternalSupplySlackDto]>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PowerSummaryDto {
    pub external_production_w: u32,
    pub external_consumption_w: u32,
    pub thermal_generation_w: u32,
    pub machine_consumption_w: u32,
    pub total_gen_w: u32,
    pub total_use_w: u32,
    pub margin_w: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OutpostValueDto {
    pub key: Box<str>,
    pub name: Box<str>,
    pub value_per_min: f64,
    pub cap_per_min: f64,
    pub ratio: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaleValueDto {
    pub outpost_key: Box<str>,
    pub outpost_name: Box<str>,
    pub item_key: Box<str>,
    pub item_name: Box<str>,
    pub qty_per_min: f64,
    pub value_per_min: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FacilityUsageDto {
    pub key: Box<str>,
    pub name: Box<str>,
    pub machines: u32,
    pub power_w: u32,
    pub total_power_w: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSupplySlackDto {
    pub item_key: Box<str>,
    pub item_name: Box<str>,
    pub supply_per_min: f64,
    pub slack_per_min: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogisticsGraphDto {
    pub items: Box<[LogisticsItemSummaryDto]>,
    pub nodes: Box<[LogisticsNodeDto]>,
    pub edges: Box<[LogisticsEdgeDto]>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogisticsItemSummaryDto {
    pub item_key: Box<str>,
    pub item_name: Box<str>,
    pub edge_count: usize,
    pub node_count: usize,
    pub total_flow_per_min: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogisticsNodeDto {
    pub id: Box<str>,
    pub kind: Box<str>,
    pub label: Box<str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogisticsEdgeDto {
    pub id: Box<str>,
    pub item_key: Box<str>,
    pub item_name: Box<str>,
    pub source: Box<str>,
    pub target: Box<str>,
    pub flow_per_min: f64,
}

#[derive(Debug, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub(crate) enum ApiEnvelope<T> {
    Ok { data: T },
    Err { error: ApiErrorDto },
}

#[derive(Debug, Serialize)]
pub(crate) struct ApiErrorDto {
    message: Box<str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source: Option<Box<str>>,
}

fn error_source_chain(err: &crate::Error) -> Option<Box<str>> {
    let mut source = err.source();
    let mut chain = Vec::<String>::new();

    while let Some(next) = source {
        chain.push(next.to_string());
        source = next.source();
    }

    if chain.is_empty() {
        None
    } else {
        Some(chain.join(": ").into_boxed_str())
    }
}

pub(crate) fn envelope_json<T: Serialize>(result: Result<T>) -> String {
    let envelope = match result {
        Ok(data) => ApiEnvelope::Ok { data },
        Err(err) => ApiEnvelope::Err {
            error: ApiErrorDto {
                message: err.to_string().into_boxed_str(),
                source: error_source_chain(&err),
            },
        },
    };

    match serde_json::to_string(&envelope) {
        Ok(json) => json,
        Err(_) => "{\"status\":\"err\",\"error\":{\"message\":\"failed to serialize response\"}}"
            .to_string(),
    }
}
