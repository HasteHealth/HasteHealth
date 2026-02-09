use etl::destination::Destination;
use etl::error::EtlResult;
use etl::types::{Cell, Event, TableId, TableRow};
use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_fhir_search::{IndexResource, SearchEngine};
use haste_jwt::{ProjectId, ResourceId, TenantId, VersionId};
use haste_repository::types::{FHIRMethod, SupportedFHIRVersions};
use tracing::info;

// Important
// Column Order is as follows (defined by schema)
// 0  id             | text                     |           | not null | generated always as (resource ->> 'id'::text) stored
// 1  tenant         | text                     |           | not null |
// 2  project        | text                     |           | not null |
// 3  resource_type  | text                     |           | not null | generated always as (resource ->> 'resourceType'::text) stored
// 4  author_id      | text                     |           | not null |
// 5  resource       | jsonb                    |           | not null |
// 6  deleted        | boolean                  |           | not null | false
// 7  created_at     | timestamp with time zone |           | not null | now()
// 8  request_method | character varying(7)     |           |          | 'PUT'::character varying
// 9  fhir_version   | fhir_version             |           | not null |
// 10 author_type    | text                     |           | not null |
// 11 version_id     | text                     |           | not null | generated always as ((resource -> 'meta'::text) ->> 'versionId'::text) stored
// 12 fhir_method    | fhir_method              |           | not null |
// 13 sequence       | bigint                   |           | not null | nextval('resources_sequence_seq'::regclass)

#[derive(Debug, Clone)]
pub struct ESSearchDestination<Search: SearchEngine> {
    search_client: Search,
}

impl<Search: SearchEngine> ESSearchDestination<Search> {
    pub fn new(search_client: Search) -> EtlResult<Self> {
        Ok(Self { search_client })
    }
}

impl<Search: SearchEngine> Destination for ESSearchDestination<Search> {
    fn name() -> &'static str {
        "http"
    }

    async fn truncate_table(&self, _table_id: TableId) -> EtlResult<()> {
        todo!("Not implemented")
    }

    async fn write_table_rows(&self, _table_id: TableId, _rows: Vec<TableRow>) -> EtlResult<()> {
        todo!("Not implemented")
    }

    async fn write_events(&self, events: Vec<Event>) -> EtlResult<()> {
        if events.is_empty() {
            return Ok(());
        }
        info!("Writing {} events", events.len());

        let indexed_resources = events
            .into_iter()
            .filter_map(|e| {
                if let Event::Insert(i) = e {
                    Some(i.table_row.values)
                } else {
                    None
                }
            })
            .map(|mut i| IndexResource {
                id: match i.swap_remove(0) {
                    Cell::String(id) => ResourceId::new(id),
                    _ => {
                        panic!("Unexpected cell type for id: {:?}", i[0]);
                    }
                },
                tenant: match i.swap_remove(1) {
                    Cell::String(tenant) => TenantId::new(tenant),
                    _ => {
                        panic!("Unexpected cell type for tenant: {:?}", i[1]);
                    }
                },
                version_id: match i.swap_remove(11) {
                    Cell::String(version_str) => VersionId::new(version_str),
                    _ => {
                        panic!("Unexpected cell type for project: {:?}", i[2]);
                    }
                },
                project: match i.swap_remove(2) {
                    Cell::String(project) => ProjectId::new(project),
                    _ => {
                        panic!("Unexpected cell type for project: {:?}", i[2]);
                    }
                },
                fhir_method: match i.swap_remove(12) {
                    Cell::String(fhir_method) => {
                        FHIRMethod::try_from(fhir_method.as_str()).unwrap()
                    }
                    _ => {
                        panic!("Unexpected cell type for fhir_method: {:?}", i[12]);
                    }
                },
                resource_type: match i.swap_remove(3) {
                    Cell::String(text) => ResourceType::try_from(text).unwrap(),
                    _ => {
                        panic!("Unexpected cell type for resource_type: {:?}", i[3]);
                    }
                },
                resource: match i.swap_remove(5) {
                    Cell::Json(json) => {
                        haste_fhir_serialization_json::from_serde_value(json).unwrap()
                    }
                    _ => {
                        panic!("Unexpected cell type for resource: {:?}", i[5]);
                    }
                },
            })
            .collect::<Vec<_>>();

        self.search_client
            .index(SupportedFHIRVersions::R4, indexed_resources)
            .await
            .expect("Failed to index resources in search engine");

        Ok(())
    }
}
