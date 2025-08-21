use oxidized_indexing_worker::run_worker;

#[tokio::main]
pub async fn main() {
    run_worker().await;
}
