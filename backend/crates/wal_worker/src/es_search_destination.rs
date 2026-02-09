use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_fhir_search::{IndexResource, SearchEngine};
use haste_jwt::{ProjectId, ResourceId, VersionId};
use haste_repository::types::FHIRMethod;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, warn};

use etl::destination::Destination;
use etl::error::{ErrorKind, EtlResult};
use etl::types::{Cell, Event, TableId, TableRow};
use etl::{bail, etl_error};

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
pub struct SearchDestination<Search: SearchEngine> {
    search_client: Arc<Search>,
}

impl<Search: SearchEngine> SearchDestination<Search> {
    pub fn new(search_client: Arc<Search>) -> EtlResult<Self> {
        Ok(Self { search_client })
    }
}

impl<Search: SearchEngine> Destination for SearchDestination<Search> {
    fn name() -> &'static str {
        "http"
    }

    async fn truncate_table(&self, table_id: TableId) -> EtlResult<()> {
        todo!("Not implemented")
    }

    async fn write_table_rows(&self, table_id: TableId, rows: Vec<TableRow>) -> EtlResult<()> {
        todo!("Not implemented")
    }

    async fn write_events(&self, events: Vec<Event>) -> EtlResult<()> {
        if events.is_empty() {
            return Ok(());
        }
        info!("Writing {} events", events.len());

        for event in &events {
            match event {
                Event::Insert(i) => {
                    for v in i.table_row.values.iter() {}
                    info!("Insert into table {}", i.table_id.0)
                }
                Event::Update(u) => panic!("Unexpected Update event: {:?}", u),
                Event::Delete(d) => panic!("Unexpected Delete event: {:?}", d),
                _ => {
                    println!("Ignoring Event: {:?}", event);
                }
            }
            info!("Event: {:?}", event);
        }

        let indexed_resources = events
            .into_iter()
            .filter_map(|e| {
                if let Event::Insert(i) = e {
                    Some(i.table_row.values)
                } else {
                    None
                }
            })
            .map(|i| IndexResource {
                id: match i[0] {
                    Cell::String(id) => ResourceId::new(id),
                    _ => {
                        panic!("Unexpected cell type for id: {:?}", i[0]);
                    }
                },
                version_id: match i[11] {
                    Cell::String(version_str) => version_str,
                    _ => {
                        panic!("Unexpected cell type for project: {:?}", i[2]);
                    }
                },
                project: match i[2] {
                    Cell::String(project) => ProjectId::new(project),
                    _ => {
                        panic!("Unexpected cell type for project: {:?}", i[2]);
                    }
                },
                fhir_method: match i[12] {
                    Cell::String(fhir_method) => {
                        FHIRMethod::try_from(fhir_method.as_str()).unwrap()
                    }
                    _ => {
                        panic!("Unexpected cell type for fhir_method: {:?}", i[12]);
                    }
                },
                resource_type: match i[3] {
                    Cell::String(text) => ResourceType::try_from(text).unwrap(),
                    _ => {
                        panic!("Unexpected cell type for resource_type: {:?}", i[3]);
                    }
                },
                resource: match i[5] {
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
            .index(
                haste_fhir_model::r4::generated::resources::SupportedFHIRVersions::R4,
                "tenant_id_placeholder".to_string(),
                indexed_resources,
            )
            .await?;

        Ok(())
    }
}
