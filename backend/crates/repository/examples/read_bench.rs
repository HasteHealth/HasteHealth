//! Times the repository's multi-row reads against a seeded database. Prints
//! the median wall time per call and, per path, the total serialized size of
//! what was read (the same before and after a change that only alters how
//! rows are decoded).
//!
//! ```sh
//! DATABASE_URL=postgresql://postgres:postgres@127.0.0.1/haste_health \
//!     cargo run --release -p haste-repository --example read_bench
//! ```
use haste_fhir_client::request::{FHIRHistoryTypeRequest, HistoryRequest};
use haste_fhir_client::url::ParsedParameters;
use haste_fhir_model::r4::generated::resources::{Resource, ResourceType};
use haste_jwt::{ProjectId, ResourceId, TenantId, VersionId};
use haste_repository::{
    fhir::{CachePolicy, FHIRRepository},
    pg::PGConnection,
    sequence::ResourceSequential,
};
use std::time::Instant;

#[allow(clippy::cast_precision_loss)]
fn median(mut times: Vec<u128>) -> f64 {
    times.sort_unstable();
    times[times.len() / 2] as f64 / 1e6
}

fn size(resource: &Resource) -> usize {
    serde_json::to_vec(resource).unwrap().len()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let repo = PGConnection::pool(pool);
    let tenant = TenantId::new("bench".to_string()).unwrap();
    let project = ProjectId::new("bench".to_string()).unwrap();

    // GET Patient/<id>
    let (mut times, mut bytes) = (Vec::new(), 0);
    for i in 0..6000u32 {
        let id = ResourceId::new(format!("p{}", (i * 7919) % 20000));
        let start = Instant::now();
        let found = repo
            .read_latest(&tenant, &project, &ResourceType::Patient, &id)
            .await
            .unwrap();
        times.push(start.elapsed().as_nanos());
        bytes += size(&found.unwrap());
    }
    println!("read_latest {:.4} ms ({bytes} bytes)", median(times));

    // GET Patient/_history, the default page of 1000
    let request = HistoryRequest::Type(FHIRHistoryTypeRequest {
        resource_type: ResourceType::Patient,
        parameters: ParsedParameters::new(vec![]),
    });
    let (mut times, mut bytes) = (Vec::new(), 0);
    for _ in 0..60 {
        let start = Instant::now();
        let page = repo.history(&tenant, &project, &request).await.unwrap();
        times.push(start.elapsed().as_nanos());
        assert_eq!(page.len(), 1000);
        bytes += page.iter().map(|v| size(&v.resource)).sum::<usize>();
    }
    println!("history_1000 {:.3} ms ({bytes} bytes)", median(times));

    // the indexing worker's poll, 100 rows
    let (mut times, mut bytes) = (Vec::new(), 0);
    for _ in 0..300 {
        let start = Instant::now();
        let rows = repo
            .get_sequence(&tenant, &project, 0, i64::MAX, Some(100))
            .await
            .unwrap();
        times.push(start.elapsed().as_nanos());
        assert_eq!(rows.len(), 100);
        bytes += rows.iter().map(|v| size(&v.resource.0)).sum::<usize>();
    }
    println!("get_sequence_100 {:.3} ms ({bytes} bytes)", median(times));

    // the resources of one search page
    for n in [20usize, 100, 1000] {
        let (mut times, mut bytes) = (Vec::new(), 0);
        for i in 0..200usize {
            let ids: Vec<VersionId> = (0..n)
                .map(|k| VersionId::new(format!("pv{}", 1 + (i * 1000 + k * 37) % 40000)))
                .collect();
            let refs: Vec<&VersionId> = ids.iter().collect();
            let start = Instant::now();
            let found = repo
                .read_by_version_ids(&tenant, &project, &refs, CachePolicy::NoCache)
                .await
                .unwrap();
            times.push(start.elapsed().as_nanos());
            assert_eq!(found.len(), n);
            bytes += found.iter().map(size).sum::<usize>();
        }
        println!(
            "read_by_version_ids_{n} {:.3} ms ({bytes} bytes)",
            median(times)
        );
    }
}
