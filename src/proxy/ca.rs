use std::fs;
use std::path::PathBuf;
use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};
use time::{Duration, OffsetDateTime};

pub struct CertificateAuthority {
    cert_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CaInfo {
    pub id: String,
    pub domain: String,
    pub type_: String,
    pub issuer: String,
    pub validity: String,
}

impl CertificateAuthority {
    pub fn new() -> Self {
        let mut cert_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        cert_dir.push(".gemini");
        cert_dir.push("antigravity");
        cert_dir.push("certs");

        if !cert_dir.exists() {
            fs::create_dir_all(&cert_dir).unwrap_or_else(|e| {
                eprintln!("Failed to create cert directory: {}", e);
            });
        }

        Self { cert_dir }
    }

    pub fn get_cert_dir(&self) -> PathBuf {
        self.cert_dir.clone()
    }

    pub fn generate_ca_cert(&self) -> Result<CaInfo, Box<dyn std::error::Error>> {
        let mut params = CertificateParams::default();
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, "Oriv Self-Signed CA");
        dn.push(DnType::OrganizationName, "Oriv Proxy");
        params.distinguished_name = dn;
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];
        
        // Validity: 1 year
        let now = OffsetDateTime::now_utc();
        let not_after = now + Duration::days(365);
        params.not_before = now;
        params.not_after = not_after;

        let key_pair = KeyPair::generate()?;
        let key_pem = key_pair.serialize_pem();

        let cert = params.self_signed(&key_pair)?;
        let cert_pem = cert.pem();

        // Save to file
        let cert_path = self.cert_dir.join("ca.crt");
        let key_path = self.cert_dir.join("ca.key");

        fs::write(&cert_path, &cert_pem)?;
        fs::write(&key_path, &key_pem)?;

        println!("CA Certificate generated at: {:?}", cert_path);

        Ok(CaInfo {
            id: "ca-root".to_string(),
            domain: "Oriv Root CA".to_string(),
            type_: "自签名".to_string(),
            issuer: "Oriv Proxy".to_string(),
            validity: format!("{} 至 {}", 
                now.date(), 
                not_after.date()
            ),
        })
    }

    pub fn load_ca_info(&self) -> Option<CaInfo> {
        let cert_path = self.cert_dir.join("ca.crt");
        if !cert_path.exists() {
            return None;
        }

        // In a real app, we would parse the existing cert to get details.
        // For now, we'll return a placeholder if the file exists, 
        // or we could store metadata in a separate JSON file.
        // To keep it simple for this iteration, we'll return generic info if file exists.
        // A better approach would be to parse the PEM.
        
        Some(CaInfo {
            id: "ca-root".to_string(),
            domain: "Oriv Root CA".to_string(),
            type_: "自签名".to_string(),
            issuer: "Oriv Proxy".to_string(),
            validity: "已安装".to_string(), // TODO: Parse actual validity
        })
    }
}
