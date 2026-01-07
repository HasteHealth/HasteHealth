use haste_fhir_client::request::FHIRRequest;
use haste_jwt::claims::SubscriptionTier;

static INVOCATION_POINTS: u32 = 100;
static WRITE_POINTS: u32 = 100;
static SEARCH_POINTS: u32 = 30;
static READ_POINTS: u32 = 10;

// Per day Limits
static FREE_TIER: u32 = 25000;
static PRO_TIER: u32 = 1000000;
static TEAM_TIER: u32 = 5000000;
static UNLIMITED_TIER: u32 = u32::MAX;

pub fn get_rate_limit_for_tier(tier: &SubscriptionTier) -> u32 {
    match tier {
        SubscriptionTier::Free => FREE_TIER,
        SubscriptionTier::Professional => PRO_TIER,
        SubscriptionTier::Team => TEAM_TIER,
        SubscriptionTier::Unlimited => UNLIMITED_TIER,
    }
}

pub fn points_for_operation(request: &FHIRRequest) -> u32 {
    match request {
        FHIRRequest::Read(fhirread_request) => READ_POINTS,
        FHIRRequest::VersionRead(fhirversion_read_request) => READ_POINTS,

        FHIRRequest::Create(fhircreate_request) => WRITE_POINTS,
        FHIRRequest::Update(update_request) => WRITE_POINTS,
        FHIRRequest::Patch(fhirpatch_request) => WRITE_POINTS,
        FHIRRequest::Delete(delete_request) => WRITE_POINTS,

        FHIRRequest::Capabilities => 10,
        FHIRRequest::Search(search_request) => SEARCH_POINTS,
        FHIRRequest::History(history_request) => SEARCH_POINTS,

        FHIRRequest::Invocation(invocation_request) => INVOCATION_POINTS,

        FHIRRequest::Batch(fhirbatch_request) => todo!(),
        FHIRRequest::Transaction(fhirtransaction_request) => todo!(),
    }
}
