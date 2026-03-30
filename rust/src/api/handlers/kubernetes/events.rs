//! Kubernetes Events API handlers
//!
//! Events, Metrics, Topology visualization

use axum::{
    extract::{Path, Query, State},
    Json,
};
use k8s_openapi::api::core::v1::Event;
use kube::{api::{Api, ListParams}, Client};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::api::state::AppState;
use crate::error::{Error, Result};

// ============================================================================
// Events
// ============================================================================

#[derive(Debug, Serialize)]
pub struct KubernetesEvent {
    pub name: String,
    pub namespace: String,
    pub type_: String,
    pub reason: String,
    pub message: String,
    pub count: i32,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub involved_object: InvolvedObjectSummary,
}

#[derive(Debug, Serialize)]
pub struct InvolvedObjectSummary {
    pub kind: String,
    pub name: String,
    pub api_version: Option<String>,
    pub uid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EventsQuery {
    pub namespace: Option<String>,
    pub limit: Option<i32>,
    pub field_selector: Option<String>,
}

pub async fn list_events(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Vec<KubernetesEvent>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    
    let mut lp = ListParams::default();
    lp.limit = Some(query.limit.unwrap_or(100) as u32);
    
    if let Some(selector) = query.field_selector {
        lp.field_selector = Some(selector);
    }
    
    let api: Api<Event> = Api::namespaced(client.raw().clone(), &ns);
    let event_list = api.list(&lp).await
        .map_err(|e| Error::Kubernetes(format!("Failed to list events: {}", e)))?;
    
    let events = event_list.items.iter().map(|e| {
        let involved = &e.involved_object;
        KubernetesEvent {
            name: e.metadata.name.clone().unwrap_or_default(),
            namespace: e.metadata.namespace.clone().unwrap_or_default(),
            type_: e.type_.clone().unwrap_or_default(),
            reason: e.reason.clone().unwrap_or_default(),
            message: e.message.clone().unwrap_or_default(),
            count: e.count.unwrap_or(1),
            first_seen: e.first_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            last_seen: e.last_timestamp.as_ref().map(|t| t.0.to_rfc3339()),
            involved_object: InvolvedObjectSummary {
                kind: involved.kind.clone().unwrap_or_default(),
                name: involved.name.clone().unwrap_or_default(),
                api_version: involved.api_version.clone(),
                uid: involved.uid.clone(),
            },
        }
    }).collect();
    
    Ok(Json(events))
}

pub async fn get_event(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<Event>> {
    let client = state.kubernetes_client()?;
    let api: Api<Event> = Api::namespaced(client.raw().clone(), &namespace);
    
    let event = api.get(&name).await
        .map_err(|e| Error::NotFound(format!("Event {} not found: {}", name, e)))?;
    
    Ok(Json(event))
}

// ============================================================================
// Metrics
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct PodMetrics {
    pub name: String,
    pub namespace: String,
    pub containers: Vec<ContainerMetrics>,
    pub timestamp: String,
    pub window: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContainerMetrics {
    pub name: String,
    pub usage: ResourceUsage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub name: String,
    pub usage: ResourceUsage,
    pub timestamp: String,
    pub window: String,
}

#[derive(Debug, Serialize)]
pub struct TopPods {
    pub pods: Vec<TopPodEntry>,
}

#[derive(Debug, Serialize)]
pub struct TopPodEntry {
    pub name: String,
    pub namespace: String,
    pub cpu_usage: String,
    pub memory_usage: String,
}

pub async fn get_pod_metrics(
    State(state): State<Arc<AppState>>,
    Path((namespace, name)): Path<(String, String)>,
) -> Result<Json<PodMetrics>> {
    let client = state.kubernetes_client()?;
    
    // Metrics API is separate from main Kubernetes API
    // Try to access metrics.k8s.io API group using dynamic API
    let gvk = kube::core::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics");
    let api_resource = kube::core::ApiResource::from_gvk(&gvk);
    let api: kube::Api<kube::core::DynamicObject> = kube::Api::namespaced_with(client.raw().clone(), &namespace, &api_resource);
    
    let obj = api.get(&name).await
        .map_err(|e| Error::NotFound(format!("Pod metrics not available: {}. Ensure metrics-server is installed.", e)))?;
    
    // Convert to PodMetrics
    let metrics = PodMetrics {
        name: obj.metadata.name.unwrap_or_default(),
        namespace: obj.metadata.namespace.unwrap_or_default(),
        containers: vec![],
        timestamp: String::new(),
        window: String::new(),
    };
    
    Ok(Json(metrics))
}

pub async fn list_pod_metrics(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<Vec<PodMetrics>>> {
    let client = state.kubernetes_client()?;
    let ns = query.namespace.unwrap_or_else(|| "default".to_string());
    
    let gvk = kube::core::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "PodMetrics");
    let api_resource = kube::core::ApiResource::from_gvk(&gvk);
    let api: kube::Api<kube::core::DynamicObject> = kube::Api::namespaced_with(client.raw().clone(), &ns, &api_resource);
    
    let list = api.list(&Default::default()).await
        .map_err(|e| Error::NotFound(format!("Pod metrics not available: {}", e)))?;
    
    let items: Vec<PodMetrics> = list.items.iter().map(|obj| PodMetrics {
        name: obj.metadata.name.clone().unwrap_or_default(),
        namespace: obj.metadata.namespace.clone().unwrap_or_default(),
        containers: vec![],
        timestamp: String::new(),
        window: String::new(),
    }).collect();
    
    Ok(Json(items))
}

pub async fn get_node_metrics(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<NodeMetrics>> {
    let client = state.kubernetes_client()?;
    
    let gvk = kube::core::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "NodeMetrics");
    let api_resource = kube::core::ApiResource::from_gvk(&gvk);
    let api: kube::Api<kube::core::DynamicObject> = kube::Api::all_with(client.raw().clone(), &api_resource);
    
    let obj = api.get(&name).await
        .map_err(|e| Error::NotFound(format!("Node metrics not available: {}", e)))?;
    
    let metrics = NodeMetrics {
        name: obj.metadata.name.unwrap_or_default(),
        usage: ResourceUsage { cpu: String::new(), memory: String::new() },
        timestamp: String::new(),
        window: String::new(),
    };
    
    Ok(Json(metrics))
}

pub async fn list_node_metrics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<NodeMetrics>>> {
    let client = state.kubernetes_client()?;
    
    let gvk = kube::core::GroupVersionKind::gvk("metrics.k8s.io", "v1beta1", "NodeMetrics");
    let api_resource = kube::core::ApiResource::from_gvk(&gvk);
    let api: kube::Api<kube::core::DynamicObject> = kube::Api::all_with(client.raw().clone(), &api_resource);
    
    let list = api.list(&Default::default()).await
        .map_err(|e| Error::NotFound(format!("Node metrics not available: {}", e)))?;
    
    let items: Vec<NodeMetrics> = list.items.iter().map(|obj| NodeMetrics {
        name: obj.metadata.name.clone().unwrap_or_default(),
        usage: ResourceUsage { cpu: String::new(), memory: String::new() },
        timestamp: String::new(),
        window: String::new(),
    }).collect();
    
    Ok(Json(items))
}

pub async fn get_top_pods(
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventsQuery>,
) -> Result<Json<TopPods>> {
    // For now, return empty list - real implementation requires metrics-server
    Ok(Json(TopPods { pods: vec![] }))
}

pub async fn get_top_nodes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<NodeMetrics>>> {
    // For now, return empty list - real implementation requires metrics-server
    Ok(Json(vec![]))
}

// Helper functions for parsing resource values
fn parse_cpu(cpu: &str) -> Option<u64> {
    if cpu.ends_with('m') {
        cpu[..cpu.len()-1].parse::<u64>().ok()
    } else {
        cpu.parse::<u64>().ok().map(|v| v * 1000)
    }
}

fn parse_memory(mem: &str) -> Option<u64> {
    if mem.ends_with("Ki") {
        mem[..mem.len()-2].parse::<u64>().ok().map(|v| v * 1024)
    } else if mem.ends_with("Mi") {
        mem[..mem.len()-2].parse::<u64>().ok().map(|v| v * 1024 * 1024)
    } else if mem.ends_with("Gi") {
        mem[..mem.len()-2].parse::<u64>().ok().map(|v| v * 1024 * 1024 * 1024)
    } else {
        mem.parse::<u64>().ok()
    }
}

fn format_cpu(cpu: u64) -> String {
    if cpu >= 1000 {
        format!("{}m", cpu)
    } else {
        format!("{}", cpu)
    }
}

fn format_memory(mem: u64) -> String {
    if mem >= 1024 * 1024 * 1024 {
        format!("{:.1}Gi", mem as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if mem >= 1024 * 1024 {
        format!("{:.0}Mi", mem as f64 / (1024.0 * 1024.0))
    } else if mem >= 1024 {
        format!("{:.0}Ki", mem as f64 / 1024.0)
    } else {
        format!("{}", mem)
    }
}
