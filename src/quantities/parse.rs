//! This module provides the parsing functionality for serving quantities.

use super::Quantity;

use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{char, digit1, multispace0, multispace1};
use nom::character::is_alphabetic;
use nom::combinator::{eof, iterator, map_opt, opt};
use nom::error::{Error, ErrorKind};
use nom::number::complete::float;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::{Err, IResult, Parser};

/// Parse a fraction string like `"1/2"` to the corresponding float.
fn fraction(input: &str) -> IResult<&str, f32> {
    let digit_pair = tuple((
        digit1,
        delimited(multispace0, char('/'), multispace0),
        terminated(digit1, multispace0),
    ));
    map_opt(digit_pair, |(str0, _, str1): (&str, char, &str)| {
        let num0 = str0.parse::<f32>().ok()?;
        let num1 = str1.parse::<f32>().ok()?;
        Some(num0 / num1)
    })(input)
}

/// Parse a compound fraction string like `"1 1/2"` to the corresponding float.
fn compound_fraction(input: &str) -> IResult<&str, f32> {
    map_opt(
        tuple((digit1, multispace1, fraction)),
        |(whole, _, frac): (&str, &str, f32)| whole.parse::<f32>().ok().map(|n| n + frac),
    )(input)
}

/// Parse any numeric string like `"3/2"`, `"1 1/2"`, or `"1.5"` to the corresponding float.
pub fn number(input: &str) -> IResult<&str, f32> {
    alt((compound_fraction, fraction, float))(input)
}

/// This is a simple parser that allows for words to have inter-hyphens and terminating
/// periods, as is usually the case with unit names.
pub fn unit_word<'a>(input: &'a str) -> IResult<&'a str, &'a str> {
    let opt_split_index = input.as_bytes().iter().enumerate().find_map(|(i, b)| {
        let c = char::from(*b);
        if is_alphabetic(*b) || ((c == '.' || c == '-') && i != 0) {
            None
        } else {
            Some(i)
        }
    });
    match opt_split_index {
        Some(i) if i == 0 => Err(Err::Error(Error::new(input, ErrorKind::Alpha))),
        Some(i) => Ok((&input[i..], &input[..i])),
        None => Ok(("", input)),
    }
}

/// Parser for a food quantity. It is achieved by first matching on a numeric value and
/// iteratively grabbing words until the resulting string matches an SI unit or it can grab no
/// more. In the latter case, it returns the [`Quantity::Nominal`] variant.
pub fn quantity<'a>(input: &'a str) -> IResult<&'a str, Quantity> {
    // any quantity must be a number and at least one word
    let number_space = terminated(number, multispace0);
    let mut required = tuple((number_space, unit_word));
    match required.parse(input) {
        // if we cannot match "number word", then we consider the parser failed
        Err(e) => Err(e),
        // otherwise, we check if "word" is associated to some si unit
        Ok((input, (val, word))) => match units::si_quantity(val.clone(), word) {
            // if so, return the quantity
            Some(quantity) => Ok((input, quantity)),
            // if not, continue grabbing words
            None => {
                // a string buffer will hold the words we see
                let mut words = String::with_capacity(256);
                words.push_str(&word.to_lowercase());

                // create an iterator which will repeatedly grab words, checking if the
                // sequence of words is associated to an SI unit. If we run out of words before
                // seeing an SI unit, we consider it nominal.
                let mut iter = iterator(input, preceded(multispace1, unit_word));
                let quantity = iter
                    .scan(&mut words, |words, word| {
                        words.push_str(" ");
                        words.push_str(&word.to_lowercase());
                        Some(units::si_quantity(val.clone(), words))
                    })
                    .find_map(|opt_quant| opt_quant)
                    .unwrap_or(Quantity::Nominal(val, words));
                let (input, _) = iter.finish()?;
                Ok((input, quantity))
            }
        },
    }
}

pub fn noise<'a>(input: &'a str) -> IResult<&'a str, ()> {
    let mut iter = iterator::<&'a str, &'a str, Error<&'a str>, _>(
        input,
        alt((
            tag_no_case("about"),
            tag_no_case("approx."),
            tag_no_case("approximately"),
            tag_no_case("makes"),
            tag("\""),
            tag("|"),
            multispace1,
        )),
    );
    iter.for_each(|_| {});
    match iter.finish() {
        Ok(o) => Ok(o),
        Err(_) => Ok((input, ())),
    }
}

/// Parser for the food quantities on a label. Implemented by stripping artifacts and repeatedly
/// applying the [`quantity`] parser.
pub fn quantities<'a>(input: &'a str) -> IResult<&'a str, Vec<Quantity>> {
    // first run a parse on a single quantity
    let res = delimited(noise, quantity, multispace0)(input);
    match res {
        Err(e) => Err(e),
        Ok((input, q)) => {
            // if we saw a quantity, continue to parse until eof, allowing parentheses
            let mut iter = iterator(
                input,
                delimited(
                    multispace0,
                    delimited(
                        opt(tag("(")),
                        delimited(noise, quantity, noise),
                        opt(tag(")")),
                    ),
                    multispace0,
                ),
            );
            let mut quants = iter.collect::<Vec<_>>();
            quants.push(q);
            let (input, _) = iter.finish()?;
            let _ = preceded(noise, eof)(input)?;
            Ok((input, quants))
        }
    }
}

/// This module simply holds static variables which are used for parsing units
mod units {
    use super::Quantity;
    use uom::si::{
        f32::{Mass, Volume},
        mass::{self, centigram, gram, kilogram, milligram, ounce, pound},
        volume::{
            self, centiliter, cubic_centimeter, cubic_inch, cup, fluid_ounce, gallon, liter,
            milliliter, pint_liquid, quart_liquid, tablespoon, teaspoon,
        },
    };

    /// We store all the units in an enum to ensure each one is matched against
    #[allow(non_camel_case_types)]
    enum Units {
        CENTILITER,
        CUBIC_CENTIMETER,
        CUBIC_INCH,
        CUP,
        FLUID_OUNCE,
        GALLON,
        LITER,
        MILLILITER,
        PINT,
        QUART,
        TABLESPOON,
        TEASPOON,
        CENTIGRAM,
        GRAM,
        KILOGRAM,
        MILLIGRAM,
        OUNCE,
        POUND,
        NONE,
    }

    /// Map various names associated to a unit to a normalized static candidate. A [`None`]
    /// variant corresponds to the input string slice not being associated to a unit.
    fn normalize_unit<'a>(input: &'a str) -> Units {
        match &input.to_lowercase()[..] {
            // volumes
            "centiliter" | "centiliters" | "cl" => Units::CENTILITER,
            "cubic centimeter" | "cubic centimeters" => Units::CUBIC_CENTIMETER,
            "cubic inch" | "cubic inches" => Units::CUBIC_INCH,
            "cup" | "cups" => Units::CUP,
            "fl.oz." | "fl. oz." | "fl oz" | "fluid ounce" | "fluid oz" | "fluid ounces"
            | "oza" => Units::FLUID_OUNCE,
            "gallon" | "gallons" | "gals" | "gal" => Units::GALLON,
            "l" | "liter" | "liters" => Units::LITER,
            "ml" | "milliliter" | "milliliters" => Units::MILLILITER,
            "pint" | "pints" => Units::PINT,
            "quart" | "quarts" => Units::QUART,
            "tbsp" | "tablespoon" | "tablespoons" => Units::TABLESPOON,
            "tsp" | "teaspoon" | "teaspoons" => Units::TEASPOON,
            // masses
            "centigram" | "centigrams" | "cg" => Units::CENTIGRAM,
            "gram" | "grams" | "g" | "grm" | "gr" => Units::GRAM,
            "kilogram" | "kilograms" | "kg" => Units::KILOGRAM,
            "milligram" | "milligrams" | "mg" => Units::MILLIGRAM,
            "ounce" | "onz" | "ounces" | "oz" | "oz." | "wt. oz." | "wt.oz." | "wt oz" => {
                Units::OUNCE
            }
            "pound" | "pounds" | "lb" | "lbs" => Units::POUND,
            // no match
            &_ => Units::NONE,
        }
    }

    /// helper function which creates volume quantities
    fn v<U>(amount: f32) -> Quantity
    where
        U: volume::Unit + volume::Conversion<f32, T = f32>,
    {
        Quantity::Volume(Volume::new::<U>(amount))
    }

    /// helper function which creates mass quantities
    fn m<U>(amount: f32) -> Quantity
    where
        U: mass::Unit + mass::Conversion<f32, T = f32>,
    {
        Quantity::Mass(Mass::new::<U>(amount))
    }

    impl Units {
        /// helper function which creates quantities
        fn si_quantity(&self, amount: f32) -> Option<Quantity> {
            match self {
                Units::CENTILITER => Some(v::<centiliter>(amount)),
                Units::CUBIC_CENTIMETER => Some(v::<cubic_centimeter>(amount)),
                Units::CUBIC_INCH => Some(v::<cubic_inch>(amount)),
                Units::CUP => Some(v::<cup>(amount)),
                Units::FLUID_OUNCE => Some(v::<fluid_ounce>(amount)),
                Units::GALLON => Some(v::<gallon>(amount)),
                Units::LITER => Some(v::<liter>(amount)),
                Units::MILLILITER => Some(v::<milliliter>(amount)),
                Units::PINT => Some(v::<pint_liquid>(amount)),
                Units::QUART => Some(v::<quart_liquid>(amount)),
                Units::TABLESPOON => Some(v::<tablespoon>(amount)),
                Units::TEASPOON => Some(v::<teaspoon>(amount)),
                Units::CENTIGRAM => Some(m::<centigram>(amount)),
                Units::GRAM => Some(m::<gram>(amount)),
                Units::KILOGRAM => Some(m::<kilogram>(amount)),
                Units::MILLIGRAM => Some(m::<milligram>(amount)),
                Units::OUNCE => Some(m::<ounce>(amount)),
                Units::POUND => Some(m::<pound>(amount)),
                Units::NONE => None,
            }
        }
    }

    /// helper function which creates si quantities
    pub fn si_quantity(amount: f32, input: &str) -> Option<Quantity> {
        normalize_unit(input).si_quantity(amount)
    }
}
