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
    #[serde(rename = "pagingMetadata")]
    pub paging_metadata: Option<PagingMetadata>,
    pub notices: Option<Vec<Notice>>,
    pub links: Option<Vec<Link>>,
}

#[derive(Deserialize, Debug)]
pub struct EntitySearchResponse {
    #[serde(rename = "entitySearchResults")]
    pub results: Option<Vec<EntityResponse>>,
    #[serde(rename = "pagingMetadata")]
    pub paging_metadata: Option<PagingMetadata>,
    pub notices: Option<Vec<Notice>>,
    pub links: Option<Vec<Link>>,
}

#[derive(Deserialize, Debug)]
pub struct HostSearchResponse {
    #[serde(rename = "nameserverSearchResults")]
    pub results: Option<Vec<Nameserver>>,
    #[serde(rename = "pagingMetadata")]
    pub paging_metadata: Option<PagingMetadata>,
    pub notices: Option<Vec<Notice>>,
    pub links: Option<Vec<Link>>,
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
