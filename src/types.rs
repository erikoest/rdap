use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct Notice {
    pub title: Option<String>,
    pub description: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
pub struct Link {
    pub rel: Option<String>,
    pub href: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct HelpResponse {
    pub notices: Option<Vec<Notice>>,
    pub remarks: Option<Vec<Notice>>,
}

#[derive(Deserialize, Debug)]
pub struct Event {
    #[serde(rename = "eventAction")]
    pub action: String,
    #[serde(rename = "eventDate")]
    pub date: String,
}

#[derive(Deserialize, Debug)]
pub struct Nameserver {
    #[serde(rename = "ldhName")]
    pub ldh_name: Option<String>,
    pub handle: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Entity {
    pub handle: Option<String>,
    pub roles: Option<Vec<String>>,
    #[serde(rename = "vcardArray")]
    pub vcard_array: Option<Value>,
}

#[derive(Deserialize, Debug)]
pub struct DomainResponse {
    pub handle: Option<String>,
    #[serde(rename = "ldhName")]
    pub ldh_name: Option<String>,
    #[serde(rename = "unicodeName")]
    pub unicode_name: Option<String>,
    pub status: Option<Vec<String>>,
    pub events: Option<Vec<Event>>,
    pub nameservers: Option<Vec<Nameserver>>,
    pub entities: Option<Vec<Entity>>,
}

#[derive(Deserialize, Debug)]
pub struct HostResponse {
    pub handle: Option<String>,
    #[serde(rename = "startAddress")]
    pub start_address: Option<String>,
    #[serde(rename = "endAddress")]
    pub end_address: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub ip_type: Option<String>,
    pub country: Option<String>,
    pub status: Option<Vec<String>>,
    pub events: Option<Vec<Event>>,
    pub entities: Option<Vec<Entity>>,
}

#[derive(Deserialize, Debug)]
pub struct PublicId {
    #[serde(rename = "type")]
    pub id_type: Option<String>,
    pub identifier: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct EntityResponse {
    pub handle: Option<String>,
    #[serde(rename = "objectClassName")]
    pub class: Option<String>,
    pub status: Option<Vec<String>>,
    pub roles: Option<Vec<String>>,
    #[serde(rename = "vcardArray")]
    pub vcard_array: Option<Value>,
    pub events: Option<Vec<Event>>,
    pub entities: Option<Vec<Entity>>,
    #[serde(rename = "publicIds")]
    pub public_ids: Option<Vec<PublicId>>,
}

#[derive(Deserialize, Debug)]
pub struct PagingMetadata {
    #[serde(rename = "totalCount")]
    pub total_count: Option<u64>,
    #[serde(rename = "pageNumber")]
    pub page_number: Option<u64>,
    #[serde(rename = "pageSize")]
    pub page_size: Option<u64>,
    pub links: Option<Vec<Link>>,
}

#[derive(Deserialize, Debug)]
pub struct DomainSearchResponse {
    #[serde(rename = "domainSearchResults")]
    pub results: Option<Vec<DomainResponse>>,
    #[serde(rename = "paging_metadata")]
    pub paging_metadata: Option<PagingMetadata>,
    pub notices: Option<Vec<Notice>>,
    pub links: Option<Vec<Link>>,
}

#[derive(Deserialize, Debug)]
pub struct EntitySearchResponse {
    #[serde(rename = "entitySearchResults")]
    pub results: Option<Vec<EntityResponse>>,
    #[serde(rename = "paging_metadata")]
    pub paging_metadata: Option<PagingMetadata>,
    pub notices: Option<Vec<Notice>>,
    pub links: Option<Vec<Link>>,
}

#[derive(Deserialize, Debug)]
pub struct HostSearchResponse {
    #[serde(rename = "nameserverSearchResults")]
    pub results: Option<Vec<Nameserver>>,
    #[serde(rename = "paging_metadata")]
    pub paging_metadata: Option<PagingMetadata>,
    pub notices: Option<Vec<Notice>>,
    pub links: Option<Vec<Link>>,
}

#[derive(Deserialize, Debug)]
pub struct DomainCount {
    pub count: Option<u64>,
    #[serde(rename = "parentDomainName")]
    pub parent_domain_name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct NoridDomainCountResponse {
    pub domain_count: Option<Vec<DomainCount>>,
}

pub fn vcard_field(vcard: &Option<Value>, field: &str) -> Option<String> {
    let arr = vcard.as_ref()?.as_array()?;
    // vcardArray is ["vcard", [[type, params, kind, value], ...]]
    let entries = arr.get(1)?.as_array()?;
    for entry in entries {
        let e = entry.as_array()?;
        if e.first()?.as_str()? == field {
            return Some(e.get(3)?.as_str().unwrap_or("").to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{from_str, json};

    // ── vcard_field ───────────────────────────────────────────────────────────

    #[test]
    fn vcard_field_extracts_named_field() {
        let v = json!(["vcard", [
            ["version", {}, "text", "4.0"],
            ["fn",      {}, "text", "Jane Doe"],
            ["email",   {}, "text", "jane@example.com"]
        ]]);
        assert_eq!(vcard_field(&Some(v), "fn"), Some("Jane Doe".to_string()));
    }

    #[test]
    fn vcard_field_returns_none_for_missing_field() {
        let v = json!(["vcard", [["fn", {}, "text", "Jane Doe"]]]);
        assert_eq!(vcard_field(&Some(v), "email"), None);
    }

    #[test]
    fn vcard_field_returns_none_for_none_vcard() {
        assert_eq!(vcard_field(&None, "fn"), None);
    }

    #[test]
    fn vcard_field_returns_none_for_malformed_vcard() {
        let v = json!("not an array");
        assert_eq!(vcard_field(&Some(v), "fn"), None);
    }

    // ── DomainResponse deserialization ────────────────────────────────────────

    #[test]
    fn domain_response_deserializes_renamed_fields() {
        let json = r#"{
            "ldhName": "example.com",
            "unicodeName": "example.com",
            "handle": "D1234",
            "status": ["active"],
            "events": [{"eventAction": "registration", "eventDate": "2020-01-01T00:00:00Z"}],
            "nameservers": [{"ldhName": "ns1.example.com"}]
        }"#;
        let r: DomainResponse = from_str(json).unwrap();
        assert_eq!(r.ldh_name.as_deref(), Some("example.com"));
        assert_eq!(r.unicode_name.as_deref(), Some("example.com"));
        assert_eq!(r.handle.as_deref(), Some("D1234"));
        assert_eq!(r.status.as_deref(), Some(["active".to_string()].as_slice()));
        let ev = &r.events.unwrap()[0];
        assert_eq!(ev.action, "registration");
        assert_eq!(ev.date, "2020-01-01T00:00:00Z");
        assert_eq!(r.nameservers.unwrap()[0].ldh_name.as_deref(), Some("ns1.example.com"));
    }

    #[test]
    fn domain_response_optional_fields_default_to_none() {
        let r: DomainResponse = from_str("{}").unwrap();
        assert!(r.ldh_name.is_none());
        assert!(r.status.is_none());
        assert!(r.events.is_none());
    }

    // ── HostResponse deserialization ──────────────────────────────────────────

    #[test]
    fn host_response_deserializes_renamed_fields() {
        let json = r#"{
            "startAddress": "192.0.2.0",
            "endAddress": "192.0.2.255",
            "type": "ASSIGNED"
        }"#;
        let r: HostResponse = from_str(json).unwrap();
        assert_eq!(r.start_address.as_deref(), Some("192.0.2.0"));
        assert_eq!(r.end_address.as_deref(), Some("192.0.2.255"));
        assert_eq!(r.ip_type.as_deref(), Some("ASSIGNED"));
    }

    // ── EntityResponse deserialization ────────────────────────────────────────

    #[test]
    fn entity_response_deserializes_renamed_fields() {
        let json = r#"{"objectClassName": "entity", "handle": "GOGL"}"#;
        let r: EntityResponse = from_str(json).unwrap();
        assert_eq!(r.class.as_deref(), Some("entity"));
        assert_eq!(r.handle.as_deref(), Some("GOGL"));
    }

    // ── DomainSearchResponse / pagingMetadata ─────────────────────────────────

    #[test]
    fn domain_search_response_deserializes_paging_metadata() {
        let json = r#"{
            "domainSearchResults": [{"ldhName": "example.com"}],
            "paging_metadata": {
                "totalCount": 42,
                "pageNumber": 1,
                "pageSize": 10,
                "links": [{"rel": "next", "href": "https://rdap.org/domains?name=ex*&cursor=abc"}]
            }
        }"#;
        let r: DomainSearchResponse = from_str(json).unwrap();
        assert_eq!(r.results.as_ref().unwrap().len(), 1);
        let pm = r.paging_metadata.unwrap();
        assert_eq!(pm.total_count, Some(42));
        assert_eq!(pm.page_number, Some(1));
        assert_eq!(pm.page_size, Some(10));
        let link = &pm.links.unwrap()[0];
        assert_eq!(link.rel.as_deref(), Some("next"));
    }

    #[test]
    fn domain_search_response_paging_metadata_absent_is_none() {
        let r: DomainSearchResponse = from_str(r#"{"domainSearchResults": []}"#).unwrap();
        assert!(r.paging_metadata.is_none());
    }

    // ── EntitySearchResponse deserialization ──────────────────────────────────

    #[test]
    fn entity_search_response_deserializes_results_key() {
        let json = r#"{"entitySearchResults": [{"handle": "GOGL"}]}"#;
        let r: EntitySearchResponse = from_str(json).unwrap();
        assert_eq!(r.results.as_ref().unwrap()[0].handle.as_deref(), Some("GOGL"));
    }

    // ── HostSearchResponse deserialization ────────────────────────────────────

    #[test]
    fn host_search_response_deserializes_results_key() {
        let json = r#"{"nameserverSearchResults": [{"ldhName": "ns1.example.com"}]}"#;
        let r: HostSearchResponse = from_str(json).unwrap();
        assert_eq!(r.results.as_ref().unwrap()[0].ldh_name.as_deref(), Some("ns1.example.com"));
    }
}
