//! Contains all of the json payloads we get from the FDC API.

/// Corresponds to the base information every food has.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct AbridgedFoodItem {
    pub fdc_id: i32,
    pub data_type: String,
    pub description: String,
    pub food_nutrients: Vec<AbridgedFoodNutrient>,
}

/// Corresponds to a nutrient.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct AbridgedFoodNutrient {
    pub nutrient_id: i32,
    pub nutrient_name: String,
    pub unit_name: String,
    pub value: f32,
}

/// Corresponds to the metadata that only branded foods have.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct BrandedFoodItem {
    pub fdc_id: i32,
    pub brand_owner: Option<String>,
    pub brand_name: Option<String>,
    pub gtin_upc: Option<String>,
    pub household_serving_full_text: Option<String>,
    pub ingredients: String,
    pub serving_size: f32,
    pub serving_size_unit: String,
    pub label_nutrients: Option<LabelNutrients>,
}

/// Corresponds to label nutrients on branded foods.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct LabelNutrients {
    pub fat: LabelNutrient,
    pub saturated_fat: LabelNutrient,
    pub trans_fat: LabelNutrient,
    pub cholesterol: LabelNutrient,
    pub sodium: LabelNutrient,
    pub carbohydrates: LabelNutrient,
    pub fiber: LabelNutrient,
    pub sugars: LabelNutrient,
    pub protein: LabelNutrient,
    pub calcium: LabelNutrient,
    pub iron: LabelNutrient,
    pub potassium: LabelNutrient,
    pub calories: LabelNutrient,
}

/// Corresponds to a single nutrient's data in a branded food.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct LabelNutrient {
    pub value: f32,
}

/// Corresponds to the metadata of collections of both `FoodAttribute` and `FoodPortion` structs.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct APFoodItem {
    pub fdc_id: i32,
    pub food_attributes: Vec<FoodAttribute>,
    pub food_portions: Vec<FoodPortion>,
}

/// Corresponds to the food attributes,
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct FoodAttribute {
    pub id: i32,
    pub sequence_number: Option<i32>,
    pub value: String,
    pub food_attribute_type: FoodAttributeType,
}

/// Corresponds to metadata of a food attribute.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct FoodAttributeType {
    pub id: i32,
    pub name: String,
    pub description: String,
}

/// Corresponds to the portions of a given food.
#[derive(Debug, Deserialize)]
#[serde(rename_all(deserialize = "camelCase", serialize = "snake_case"))]
pub struct FoodPortion {
    pub id: i32,
    pub amount: Option<f32>,
    pub data_points: Option<i32>,
    pub gram_weight: f32,
    pub modifier: Option<String>,
    pub portion_description: Option<String>,
    pub sequence_number: Option<i32>,
}

/// A helper for parsing whether or not a food falls into the branded category.
#[derive(Debug, Deserialize)]
#[serde(tag = "dataType")]
pub enum FDCMeta {
    Branded(BrandedFoodItem),
    Other(APFoodItem),
}
