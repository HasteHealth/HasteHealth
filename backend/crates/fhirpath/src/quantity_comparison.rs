//! Comparing time-valued Quantities for `=`, `~`, `<`, `>`, `<=` and `>=`
//! (<https://hl7.org/fhirpath/N1/#time-valued-quantities>).
//!
//! A calendar duration keyword and its UCUM unit (`week` and `'wk'`) are the same unit. A week
//! and below are fixed multiples of a millisecond and compare after conversion. A year or month
//! has no fixed length: it only compares with the same unit, and a calendar one against a UCUM
//! one (`1 year` and `1 'a'`) is uncomparable, though equivalent when the values are equal.

use crate::{date_comparison::TemporalOrder, quantity_unit};
use haste_fhir_model::r4::generated::types::Quantity;
use haste_reflect::MetaValue;
use std::cmp::Ordering;

#[derive(Clone, Copy)]
enum Unit {
    /// Milliseconds in the unit.
    Fixed(f64),
    /// Whether the unit is a calendar keyword.
    Year(bool),
    Month(bool),
}

fn unit_of(unit: &str) -> Option<Unit> {
    Some(match unit {
        "year" | "years" => Unit::Year(true),
        "a" => Unit::Year(false),
        "month" | "months" => Unit::Month(true),
        "mo" => Unit::Month(false),
        "week" | "weeks" | "wk" => Unit::Fixed(604_800_000.0),
        "day" | "days" | "d" => Unit::Fixed(86_400_000.0),
        "hour" | "hours" | "h" => Unit::Fixed(3_600_000.0),
        "minute" | "minutes" | "min" => Unit::Fixed(60_000.0),
        "second" | "seconds" | "s" => Unit::Fixed(1_000.0),
        "millisecond" | "milliseconds" | "ms" => Unit::Fixed(1.0),
        _ => return None,
    })
}

/// Both values and units when each item is a Quantity with a time unit.
fn operands(left: &dyn MetaValue, right: &dyn MetaValue) -> Option<(f64, Unit, f64, Unit)> {
    let side = |item: &dyn MetaValue| {
        let quantity = item.as_any().downcast_ref::<Quantity>()?;
        Some((
            quantity.value.as_ref()?.value?,
            unit_of(quantity_unit(quantity))?,
        ))
    };
    let (left_value, left_unit) = side(left)?;
    let (right_value, right_unit) = side(right)?;
    Some((left_value, left_unit, right_value, right_unit))
}

/// `None` when the items are not both time-valued Quantities; `Unknown` when they cannot be
/// compared.
pub fn time_order(left: &dyn MetaValue, right: &dyn MetaValue) -> Option<TemporalOrder> {
    let (left_value, left_unit, right_value, right_unit) = operands(left, right)?;
    let ordering = match (left_unit, right_unit) {
        (Unit::Fixed(l), Unit::Fixed(r)) => (left_value * l).partial_cmp(&(right_value * r)),
        (Unit::Year(l), Unit::Year(r)) | (Unit::Month(l), Unit::Month(r)) if l == r => {
            left_value.partial_cmp(&right_value)
        }
        _ => None,
    };
    Some(ordering.map_or(TemporalOrder::Unknown, TemporalOrder::Ordered))
}

/// `None` when the items are not both time-valued Quantities.
pub fn time_equivalent(left: &dyn MetaValue, right: &dyn MetaValue) -> Option<bool> {
    let (left_value, left_unit, right_value, right_unit) = operands(left, right)?;
    Some(match (left_unit, right_unit) {
        (Unit::Fixed(l), Unit::Fixed(r)) => {
            (left_value * l).partial_cmp(&(right_value * r)) == Some(Ordering::Equal)
        }
        (Unit::Year(_), Unit::Year(_)) | (Unit::Month(_), Unit::Month(_)) => {
            left_value.partial_cmp(&right_value) == Some(Ordering::Equal)
        }
        _ => false,
    })
}
