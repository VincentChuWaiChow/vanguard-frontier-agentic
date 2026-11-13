use serde::{Deserialize, Serialize};

/// Cloud and platform providers represented in the catalog.
///
/// Uses kebab-case serialization to match the JSON catalog format.
/// The design specifies 19 core variants; additional variants are included
/// to support the full catalog which has grown beyond the original spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Provider {
    // Core 19 variants from design spec
    Aws,
    Azure,
    Oracle,
    Oci,
    Gcp,
    Alibaba,
    Huawei,
    Ovhcloud,
    Ionos,
    Scaleway,
    Hetzner,
    Contabo,
    Kubernetes,
    Terraform,
    MultiCloud,
    Generic,
    Frontend,
    Dotnet,
    Java,
    Kotlin,
    Hr,
    Legal,
    Salesforce,
    // Extended variants for full catalog support
    Accounting,
    Finance,
    Netsuite,
    Atlassian,
    Sap,
    Argocd,
    Backstage,
    CertManager,
    Cilium,
    Claude,
    Databricks,
    Falco,
    Fluxcd,
    Istio,
    Kyverno,
    Marketing,
    Microsoft,
    Nvidia,
    Opentelemetry,
    Prometheus,
    Sigstore,
    Snowflake,
    Velero,
    Php,
    Python,
    Typescript,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("{:?}", self).to_lowercase());
        write!(f, "{}", s)
    }
}
