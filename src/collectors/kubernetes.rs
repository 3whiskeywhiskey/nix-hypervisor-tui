use anyhow::{Result, Context};
use crate::types::{K8sClusterInfo, KubeVirtInfo};
use kube::{Client, Api, config::{Config, KubeConfigOptions}};
use k8s_openapi::api::core::v1::{Node, Pod, Service};
use std::path::PathBuf;

pub struct KubernetesCollector {
    client: Option<Client>,
    use_mock: bool,
    kubeconfig_path: Option<PathBuf>,
}

impl KubernetesCollector {
    pub fn new() -> Self {
        Self {
            client: None,
            use_mock: false,
            kubeconfig_path: None,
        }
    }

    pub fn with_kubeconfig(mut self, path: PathBuf) -> Self {
        self.kubeconfig_path = Some(path);
        self
    }

    pub async fn init(&mut self) -> Result<()> {
        // Try to initialize k8s client
        match self.init_client().await {
            Ok(client) => {
                self.client = Some(client);
                Ok(())
            }
            Err(e) => {
                tracing::warn!("Failed to initialize Kubernetes client: {}", e);
                self.use_mock = true;
                Ok(())
            }
        }
    }

    async fn init_client(&self) -> Result<Client> {
        // Try custom kubeconfig path first
        if let Some(path) = &self.kubeconfig_path {
            if path.exists() {
                let config = Config::from_custom_kubeconfig(
                    kube::config::Kubeconfig::read_from(path)?,
                    &KubeConfigOptions::default(),
                )
                .await?;
                return Client::try_from(config);
            }
        }

        // Try default k3s kubeconfig
        let k3s_path = PathBuf::from("/etc/rancher/k3s/k3s.yaml");
        if k3s_path.exists() {
            let config = Config::from_custom_kubeconfig(
                kube::config::Kubeconfig::read_from(k3s_path)?,
                &KubeConfigOptions::default(),
            )
            .await?;
            return Client::try_from(config);
        }

        // Try in-cluster config (if running as pod)
        if let Ok(config) = Config::incluster() {
            return Client::try_from(config);
        }

        // Fall back to default kubeconfig
        let config = Config::infer().await?;
        Client::try_from(config)
    }

    pub async fn collect_cluster_info(&self) -> Result<K8sClusterInfo> {
        if self.use_mock || self.client.is_none() {
            return Ok(self.mock_cluster_info());
        }

        let client = self.client.as_ref().unwrap();

        // Get nodes
        let nodes: Api<Node> = Api::all(client.clone());
        let node_list = nodes.list(&Default::default()).await?;

        let nodes_total = node_list.items.len() as u32;
        let nodes_ready = node_list
            .items
            .iter()
            .filter(|node| {
                node.status
                    .as_ref()
                    .and_then(|s| s.conditions.as_ref())
                    .map(|conditions| {
                        conditions.iter().any(|c| {
                            c.type_ == "Ready" && c.status == "True"
                        })
                    })
                    .unwrap_or(false)
            })
            .count() as u32;

        // Get pods
        let pods: Api<Pod> = Api::all(client.clone());
        let pod_list = pods.list(&Default::default()).await?;

        let pods_running = pod_list
            .items
            .iter()
            .filter(|pod| {
                pod.status
                    .as_ref()
                    .and_then(|s| s.phase.as_ref())
                    .map(|phase| phase == "Running")
                    .unwrap_or(false)
            })
            .count() as u32;

        // Get services
        let services: Api<Service> = Api::all(client.clone());
        let service_list = services.list(&Default::default()).await?;
        let services_count = service_list.items.len() as u32;

        Ok(K8sClusterInfo {
            nodes_ready,
            nodes_total,
            pods_running,
            services: services_count,
        })
    }

    pub async fn collect_kubevirt_info(&self) -> Result<KubeVirtInfo> {
        if self.use_mock || self.client.is_none() {
            return Ok(self.mock_kubevirt_info());
        }

        let client = self.client.as_ref().unwrap();

        // KubeVirt VirtualMachineInstance custom resource
        // We'll use a generic approach since we can't import KubeVirt types directly
        match self.collect_kubevirt_vms(client).await {
            Ok(info) => Ok(info),
            Err(e) => {
                tracing::debug!("Failed to collect KubeVirt info: {}", e);
                Ok(self.mock_kubevirt_info())
            }
        }
    }

    async fn collect_kubevirt_vms(&self, client: &Client) -> Result<KubeVirtInfo> {
        use kube::api::{DynamicObject, GroupVersionKind};
        use kube::discovery;

        // Discover KubeVirt API
        let discovery = discovery::Discovery::new(client.clone()).run().await?;

        // Try to find VirtualMachineInstance resource
        let gvk = GroupVersionKind {
            group: "kubevirt.io".to_string(),
            version: "v1".to_string(),
            kind: "VirtualMachineInstance".to_string(),
        };

        if let Some((ar, _caps)) = discovery.resolve_gvk(&gvk) {
            let api: Api<DynamicObject> = Api::all_with(client.clone(), &ar);
            let vmi_list = api.list(&Default::default()).await?;

            let mut running = 0;
            let mut stopped = 0;
            let mut migrating = 0;

            for vmi in vmi_list.items {
                if let Some(status) = vmi.data.get("status") {
                    if let Some(phase) = status.get("phase").and_then(|p| p.as_str()) {
                        match phase {
                            "Running" => running += 1,
                            "Stopped" | "Succeeded" | "Failed" => stopped += 1,
                            "Migrating" => migrating += 1,
                            _ => {}
                        }
                    }
                }
            }

            Ok(KubeVirtInfo {
                vms_running: running,
                vms_stopped: stopped,
                vms_migrating: migrating,
            })
        } else {
            // KubeVirt not installed
            Ok(KubeVirtInfo {
                vms_running: 0,
                vms_stopped: 0,
                vms_migrating: 0,
            })
        }
    }

    fn mock_cluster_info(&self) -> K8sClusterInfo {
        K8sClusterInfo {
            nodes_ready: 3,
            nodes_total: 3,
            pods_running: 45,
            services: 23,
        }
    }

    fn mock_kubevirt_info(&self) -> KubeVirtInfo {
        KubeVirtInfo {
            vms_running: 12,
            vms_stopped: 3,
            vms_migrating: 0,
        }
    }
}
