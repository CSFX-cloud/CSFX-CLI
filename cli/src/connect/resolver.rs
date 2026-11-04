use super::privileged;
use super::resolve::{dns_address, Group};

const RESOLVER_DIR: &str = "/etc/resolver";

pub fn domains(group: &Group, zone: &str) -> Vec<String> {
    vec![
        format!("{}.{}", group.name.to_ascii_lowercase(), zone),
        format!("svc.{}.internal", group.id),
    ]
}

pub fn install(group: &Group, zone: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }
    let address = dns_address(&group.internal_cidr).ok_or("resource group cidr is invalid")?;
    privileged::run(&["mkdir", "-p", RESOLVER_DIR])?;
    for domain in domains(group, zone) {
        let path = format!("{}/{}", RESOLVER_DIR, domain);
        privileged::write_file(&path, &format!("nameserver {}\n", address))?;
    }
    Ok(())
}

pub fn remove(group: &Group, zone: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }
    for domain in domains(group, zone) {
        privileged::run(&["rm", "-f", &format!("{}/{}", RESOLVER_DIR, domain)])?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn domains_cover_the_group_zone_and_the_service_names() {
        let group = Group {
            id: "99aec6da".to_string(),
            name: "Data-Staging".to_string(),
            internal_cidr: "10.20.0.0/24".to_string(),
        };
        assert_eq!(
            domains(&group, "csfx.internal"),
            vec!["data-staging.csfx.internal", "svc.99aec6da.internal"]
        );
    }
}
