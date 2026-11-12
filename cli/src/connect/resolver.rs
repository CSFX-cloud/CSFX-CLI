use super::privileged;
use super::resolve::{dns_address, Group};

const RESOLVER_DIR: &str = "/etc/resolver";

type Failure = Box<dyn std::error::Error>;

fn is_label(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub fn domains(group: &Group, zone: &str) -> Result<Vec<String>, Failure> {
    let name = group.name.to_ascii_lowercase();
    let zone = zone.to_ascii_lowercase();
    let id = group.id.to_ascii_lowercase();
    if !is_label(&name) {
        return Err("resource group name is not a valid dns label".into());
    }
    if !is_label(&id) {
        return Err("resource group id is not a valid dns label".into());
    }
    if !zone.split('.').all(is_label) {
        return Err("platform zone is not a valid dns name".into());
    }
    Ok(vec![
        format!("{}.{}", name, zone),
        format!("svc.{}.internal", id),
    ])
}

pub fn install(group: &Group, zone: &str) -> Result<(), Failure> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }
    let address = dns_address(&group.internal_cidr).ok_or("resource group cidr is invalid")?;
    let domains = domains(group, zone)?;
    privileged::run(&["mkdir", "-p", RESOLVER_DIR])?;
    for domain in domains {
        let path = format!("{}/{}", RESOLVER_DIR, domain);
        privileged::write_file(&path, &format!("nameserver {}\n", address))?;
    }
    Ok(())
}

pub fn remove(group: &Group, zone: &str) -> Result<(), Failure> {
    if !cfg!(target_os = "macos") {
        return Ok(());
    }
    for domain in domains(group, zone)? {
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
            domains(&group, "csfx.internal").unwrap(),
            vec!["data-staging.csfx.internal", "svc.99aec6da.internal"]
        );
    }

    fn group_named(name: &str) -> Group {
        Group {
            id: "99aec6da".to_string(),
            name: name.to_string(),
            internal_cidr: "10.20.0.0/24".to_string(),
        }
    }

    #[test]
    fn names_that_could_escape_the_resolver_directory_are_rejected() {
        for name in [
            "../../Library/x",
            "a/b",
            "..",
            "a..b",
            "a.b",
            "",
            "a b",
            "a\nb",
            "-/etc",
        ] {
            assert!(
                domains(&group_named(name), "csfx.internal").is_err(),
                "{name:?}"
            );
        }
    }

    #[test]
    fn zones_and_ids_must_be_plain_dns_names() {
        for zone in ["../x", "a//b", "", ".internal", "a..b", "x/y"] {
            assert!(domains(&group_named("web"), zone).is_err(), "{zone:?}");
        }
        let mut group = group_named("web");
        group.id = "../etc".to_string();
        assert!(domains(&group, "csfx.internal").is_err());
    }
}
