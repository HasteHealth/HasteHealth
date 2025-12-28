use crate::{context::PermissionLevel, engine::rule_engine::pdp::PDPError};

pub fn get_max(p1: &PermissionLevel, p2: &PermissionLevel) -> Result<PermissionLevel, PDPError> {
    let max = std::cmp::max(i8::from(p1), i8::from(p2));

    PermissionLevel::try_from(max).map_err(PDPError::InvalidPermissionLevel)
}

pub fn get_min(p1: &PermissionLevel, p2: &PermissionLevel) -> Result<PermissionLevel, PDPError> {
    let min = std::cmp::min(i8::from(p1), i8::from(p2));

    PermissionLevel::try_from(min).map_err(PDPError::InvalidPermissionLevel)
}
