//! This module allows us to perform HTTP requests to the
//! [FoodData Central](https://fdc.nal.usda.gov/index.html) API though the [`FDCService`] struct.

pub mod api;

pub use api::*;

use anyhow::Result;
use reqwest::Client;

/// `FDCService` implements the http requests to the FDC API through an Actix client.
#[derive(Clone, Debug)]
pub struct FDCService {
    pub fdc_key: String,
}

impl FDCService {
    /// generate a new FDCService
    pub fn new<S: Into<String>>(fdc_key: S) -> FDCService {
        FDCService {
            fdc_key: fdc_key.into(),
        }
    }

    /// Make a request to "v1/foods/search" and collect the first 10 results to a vector.
    pub async fn v1_foods_search<S: Into<String>>(
        &self,
        client: &Client,
        query: S,
    ) -> Result<Vec<AbridgedFoodItem>> {
        // make the request
        let body = serde_json::json!({ "query": query.into(), "pageSize": 10 });
        let mut res = client
            .post(format!(
                "https://api.nal.usda.gov/fdc/v1/foods/search?api_key={}",
                self.fdc_key
            ))
            .json(&body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // extract "foods" json array and deserialize
        Ok(serde_json::from_value(res["foods"].take())?)
    }

    /// Make a request to "v1/foods"
    pub async fn v1_foods(&self, client: &Client, fdc_ids: &[i32]) -> Result<Vec<FDCMeta>> {
        // make the request
        let body = serde_json::json!({ "fdcIds": fdc_ids, "format": "full" });
        let mut res = client
            .post(format!(
                "https://api.nal.usda.gov/fdc/v1/foods?api_key={}",
                self.fdc_key
            ))
            .json(&body)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // map the values associated to the `dataType` key so that they can match the enum variants
        res.as_array_mut().map(|foods| {
            foods
                .iter_mut()
                .for_each(|food| match food["dataType"].as_str() {
                    Some("Branded") => {}
                    _ => {
                        food["dataType"] = serde_json::Value::String("Other".into());
                    }
                })
        });

        // deserialize
        Ok(serde_json::from_value(res)?)
    }
}

#[cfg(test)]
mod test;
