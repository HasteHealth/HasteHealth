use oxidized_config::get_config;

pub fn main() {
    // Initialize the PostgreSQL connection pool
    let config = get_config("environment".into());
}
