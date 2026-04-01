//! Kubernetes API маршруты
//!
//! Модульная структура:
//! - cluster      — Cluster info, health, nodes
//! - namespaces   — Namespaces, quota, limits
//! - workloads    — Pods, Deployments, ReplicaSets, DaemonSets, StatefulSets
//! - networking   — Services, Ingress, NetworkPolicy, Gateway API
//! - config       — ConfigMaps, Secrets
//! - storage      — PV, PVC, StorageClass, Snapshots, CSI
//! - rbac         — ServiceAccounts, Roles, RoleBindings, ClusterRoles, PSA
//! - batch        — Jobs, CronJobs, PriorityClass, PDB
//! - advanced     — HPA, VPA, ResourceQuota, LimitRange, CRD, Custom Objects
//! - observability— Events, Metrics, Topology
//! - helm         — Helm repos, charts, releases
//! - integration  — Multi-cluster, Backup, GitOps, Audit, Runbook, Inventory Sync
//! - apply        — Apply manifest, Diff, Kubectl generator

mod cluster;
mod namespaces;
mod workloads;
mod networking;
mod config;
mod storage;
mod rbac;
mod batch;
mod advanced;
mod observability;
mod helm;
mod integration;
mod apply;

use axum::Router;
use std::sync::Arc;
use crate::api::state::AppState;

/// Создаёт маршруты Kubernetes API
pub fn kubernetes_routes() -> Router<Arc<AppState>> {
    cluster::cluster_routes()
        .merge(namespaces::namespaces_routes())
        .merge(workloads::workloads_routes())
        .merge(networking::networking_routes())
        .merge(config::config_routes())
        .merge(storage::storage_routes())
        .merge(rbac::rbac_routes())
        .merge(batch::batch_routes())
        .merge(advanced::advanced_routes())
        .merge(observability::observability_routes())
        .merge(helm::helm_routes())
        .merge(integration::integration_routes())
        .merge(apply::apply_routes())
}
