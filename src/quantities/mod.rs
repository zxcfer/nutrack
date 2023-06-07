//! This module declares the [`Quantity`] type to type different servings a food might have, along
//! with its associated string parsers.

pub mod parse;

use uom::si::f32::{Mass, Volume};

/// Serving quantities are either measured in volume/mass SI units or nominally.
#[derive(Debug, PartialEq)]
pub enum Quantity {
    Volume(Volume),
    Mass(Mass),
    Nominal(f32, String),
}

#[cfg(test)]
mod test;
