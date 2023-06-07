//! This module provides the [`Environment`] struct which holds all the information we need from
//! the environment.

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Environment {
    pub database_url: String,
    pub database_name: String,
    pub fdc_key: String,
}

struct PartialEnvironment {
    database_url: Option<String>,
    database_name: Option<String>,
    fdc_key: Option<String>,
}

pub fn get() -> Result<Environment> {
    dotenv::dotenv().ok();
    let penv = PartialEnvironment {
        database_url: None,
        database_name: None,
        fdc_key: None,
    };
    let penv = std::env::vars().fold(penv, |penv, (key, value)| {
        if key == "DATABASE_URL" {
            PartialEnvironment {
                database_url: Some(value),
                ..penv
            }
        } else if key == "DATABASE_NAME" {
            PartialEnvironment {
                database_name: Some(value),
                ..penv
            }
        } else if key == "FDC_KEY" {
            PartialEnvironment {
                fdc_key: Some(value),
                ..penv
            }
        } else {
            penv
        }
    });
    if penv.database_url.is_none() {
        Err(anyhow!("Environment needs DATABASE_URL value"))
    } else if penv.database_name.is_none() {
        Err(anyhow!("Environment needs DATABASE_NAME value"))
    } else if penv.fdc_key.is_none() {
        Err(anyhow!("Environment needs FDC_KEY value"))
    } else {
        Ok(Environment {
            database_url: penv.database_url.unwrap(),
            database_name: penv.database_name.unwrap(),
            fdc_key: penv.fdc_key.unwrap(),
        })
    }
}
