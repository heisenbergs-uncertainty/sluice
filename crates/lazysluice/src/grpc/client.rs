use std::path::Path;

use anyhow::{anyhow, Context, Result};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};

use crate::proto::sluice::v1::sluice_client::SluiceClient;
use crate::proto::sluice::v1::{ListTopicsRequest, Topic};

#[allow(dead_code)]
pub struct GrpcClient {
    inner: SluiceClient<Channel>,
}

impl GrpcClient {
    #[allow(dead_code)]
    pub async fn connect(
        endpoint: &str,
        tls_ca: Option<&Path>,
        tls_domain: Option<&str>,
    ) -> Result<Self> {
        let is_https = endpoint.starts_with("https://");
        let is_http = endpoint.starts_with("http://");
        if !is_https && !is_http {
            return Err(anyhow!("endpoint must start with http:// or https://"));
        }

        if is_http {
            if tls_ca.is_some() || tls_domain.is_some() {
                return Err(anyhow!(
                    "TLS flags are only valid with an https:// endpoint"
                ));
            }
            let channel = Endpoint::from_shared(endpoint.to_string())?
                .connect()
                .await
                .context("failed to connect")?;
            return Ok(Self {
                inner: SluiceClient::new(channel),
            });
        }

        // https://
        let tls_ca =
            tls_ca.ok_or_else(|| anyhow!("--tls-ca is required for https:// endpoints"))?;
        let ca_pem = std::fs::read(tls_ca)
            .with_context(|| format!("failed to read tls ca file: {}", tls_ca.display()))?;
        let ca_cert = Certificate::from_pem(ca_pem);

        let mut tls = ClientTlsConfig::new().ca_certificate(ca_cert);
        if let Some(domain) = tls_domain {
            tls = tls.domain_name(domain);
        }

        let channel = Endpoint::from_shared(endpoint.to_string())?
            .tls_config(tls)?
            .connect()
            .await
            .context("failed to connect")?;

        Ok(Self {
            inner: SluiceClient::new(channel),
        })
    }

    #[allow(dead_code)]
    pub async fn list_topics(&mut self) -> Result<Vec<Topic>> {
        let resp = self
            .inner
            .list_topics(ListTopicsRequest {})
            .await
            .context("list_topics failed")?
            .into_inner();
        Ok(resp.topics)
    }
}
