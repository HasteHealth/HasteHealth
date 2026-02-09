use etl::destination::Destination;
use etl::error::EtlResult;
use etl::types::{Cell, Event, TableId, TableRow};
use haste_fhir_model::r4::generated::resources::ResourceType;
use haste_fhir_search::{IndexResource, SearchEngine};
use haste_jwt::{ProjectId, ResourceId, TenantId, VersionId};
use haste_repository::types::{FHIRMethod, SupportedFHIRVersions};
use tracing::{info, warn};

// Important
// ETL does not support generated columns so id,resource_type,version_id are not avaiable as they are automatically extracted from the resource jsonb column.
// 0  id             | text                     |           | not null | generated always as (resource ->> 'id'::text) stored
// 3  resource_type  | text                     |           | not null | generated always as (resource ->> 'resourceType'::text) stored
// 11 version_id     | text                     |           | not null | generated always as ((resource -> 'meta'::text) ->> 'versionId'::text) stored

// Column Order is as follows (defined by schema)
// 0  tenant         | text                     |           | not null |
// 1  project        | text                     |           | not null |
// 2  author_id      | text                     |           | not null |
// 3  resource       | jsonb                    |           | not null |
// 4  deleted        | boolean                  |           | not null | false
// 5  created_at     | timestamp with time zone |           | not null | now()
// 6  request_method | character varying(7)     |           |          | 'PUT'::character varying
// 7  fhir_version   | fhir_version             |           | not null |
// 8  author_type    | text                     |           | not null |
// 9  fhir_method    | fhir_method              |           | not null |
// 10  sequence      | bigint                   |           | not null | nextval('resources_sequence_seq'::regclass)

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
        warn!(
            "truncate_table is not implemented for ESSearchDestination as it is not intended to be used for writing table rows directly. Received table_id: {:?}",
            _table_id
        );
        Ok(())
    }

    async fn write_table_rows(&self, _table_id: TableId, _rows: Vec<TableRow>) -> EtlResult<()> {
        warn!(
            "write_table_rows is not implemented for ESSearchDestination as it is not intended to be used for writing table rows directly. Received table_id: {:?} and rows: {:?}",
            _table_id, _rows
        );
        Ok(())
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
            .map(|mut i| {
                let mut tenant = Cell::Null;
                let mut project = Cell::Null;
                let mut resource = Cell::Null;
                let mut fhir_method = Cell::Null;

                std::mem::swap(&mut tenant, &mut i[0]);
                std::mem::swap(&mut project, &mut i[1]);
                std::mem::swap(&mut resource, &mut i[3]);
                std::mem::swap(&mut fhir_method, &mut i[9]);

                let tenant = match tenant {
                    Cell::String(tenant) => TenantId::new(tenant),
                    _ => {
                        panic!("Unexpected cell type for tenant: {:?}", i[0]);
                    }
                };
                let project = match project {
                    Cell::String(project) => ProjectId::new(project),
                    _ => {
                        panic!("Unexpected cell type for project: {:?}", i[1]);
                    }
                };
                let resource_json = match resource {
                    Cell::Json(json) => json,
                    _ => {
                        panic!("Unexpected cell type for resource: {:?}", i[5]);
                    }
                };
                // account for the 3 popped values
                let fhir_method = match fhir_method {
                    Cell::String(fhir_method) => {
                        FHIRMethod::try_from(fhir_method.as_str()).unwrap()
                    }
                    _ => {
                        panic!("Unexpected cell type for fhir_method: {:?}", i[10 - 3]);
                    }
                };

                let id = resource_json
                    .get("id")
                    .and_then(|js| js.as_str().map(|s| ResourceId::new(s.to_string())));
                let version_id = resource_json
                    .get("meta")
                    .and_then(|meta| meta.get("versionId"))
                    .and_then(|version| version.as_str().map(|s| VersionId::new(s.to_string())));
                let resource_type = resource_json
                    .get("resourceType")
                    .and_then(|js| js.as_str().map(|s| ResourceType::try_from(s).unwrap()));

                IndexResource {
                    id: id.expect("Failed to extract id"),
                    version_id: version_id.expect("Failed to extract version_id"),
                    tenant,
                    project,
                    fhir_method,
                    resource_type: resource_type.expect("Failed to extract resource_type"),
                    resource: haste_fhir_serialization_json::from_serde_value(resource_json)
                        .unwrap(),
                }
            })
            .collect::<Vec<_>>();

        self.search_client
            .index(SupportedFHIRVersions::R4, indexed_resources)
            .await
            .expect("Failed to index resources in search engine");

        Ok(())
    }
}
