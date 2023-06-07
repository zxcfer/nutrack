use super::*;

#[test]
fn number() {
    assert_eq!(parse::number("1.123 blah"), Ok((" blah", 1.123)));
    assert_eq!(parse::number("1.123blah"), Ok(("blah", 1.123)));
    assert_eq!(parse::number("1/2."), Ok((".", 0.5)));
    assert_eq!(parse::number("1 1/2."), Ok((".", 1.5)));
}

#[test]
fn noise_nonexistent() {
    assert_eq!(parse::noise("hello"), Ok(("hello", ())));
}

#[test]
fn noise_existent() {
    assert_eq!(parse::noise(" | ABOUT  "), Ok(("", ())));
}

mod quantity {
    use super::*;
    use uom::si::{
        f32::Volume,
        volume::{cubic_inch, fluid_ounce, gallon},
    };

    #[test]
    fn inital_pass_failed() {
        assert!(parse::quantity("83 -gallons").is_err());
        assert!(parse::quantity("some amount of stuff").is_err());
    }

    #[test]
    fn one_word_si_no_space() {
        assert_eq!(
            parse::quantity("83.1512gal of oil"),
            Ok((" of oil", Quantity::Volume(Volume::new::<gallon>(83.1512))))
        );
    }

    #[test]
    fn one_word_si_space() {
        assert_eq!(
            parse::quantity("83.1512 gallons of cheese"),
            Ok((
                " of cheese",
                Quantity::Volume(Volume::new::<gallon>(83.1512))
            ))
        );
    }

    #[test]
    fn two_words_si() {
        assert_eq!(
            parse::quantity("5.26 cubic inches of rice (35g)"),
            Ok((
                " of rice (35g)",
                Quantity::Volume(Volume::new::<cubic_inch>(5.26))
            ))
        );
    }

    #[test]
    fn non_alphabetic_si() {
        assert_eq!(
            parse::quantity("5.26 fl. oz. of rice (35g)"),
            Ok((
                " of rice (35g)",
                Quantity::Volume(Volume::new::<fluid_ounce>(5.26))
            ))
        );
        assert_eq!(
            parse::quantity("5.26 fl.oz. of rice (35g)"),
            Ok((
                " of rice (35g)",
                Quantity::Volume(Volume::new::<fluid_ounce>(5.26))
            ))
        );
        assert_eq!(
            parse::quantity("5.26 fl.    oz. of rice (35g)"),
            Ok((
                " of rice (35g)",
                Quantity::Volume(Volume::new::<fluid_ounce>(5.26))
            ))
        );
    }

    #[test]
    fn one_word_nominal() {
        assert_eq!(
            parse::quantity("1 package (23g Kernels)"),
            Ok((
                " (23g Kernels)",
                Quantity::Nominal(1.0, "package".to_string())
            ))
        )
    }

    #[test]
    fn two_words_nominal() {
        assert_eq!(
            parse::quantity("1 large bag (3 pounds)"),
            Ok((
                " (3 pounds)",
                Quantity::Nominal(1.0, "large bag".to_string()),
            ))
        )
    }

    #[test]
    fn non_alphabetic_nominal() {
        assert_eq!(
            parse::quantity("4.12 k-cups"),
            Ok(("", Quantity::Nominal(4.12, "k-cups".to_string()))),
        );
    }
}
