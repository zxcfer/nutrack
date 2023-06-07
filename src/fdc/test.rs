use crate::{
    env,
    fdc::{FDCMeta, FDCService},
};

fn get_service() -> FDCService {
    let environment = env::get().unwrap();
    FDCService::new(environment.fdc_key)
}

#[tokio::test]
#[ignore]
async fn v1_foods_search() {
    // get the service and a client
    let service = get_service();
    let client = reqwest::Client::new();

    // first search is a upc:
    let mut results = service
        .v1_foods_search(&client, "00027000690260")
        .await
        .unwrap();
    let unique = results.pop().unwrap();
    assert!(results.is_empty());
    assert_eq!(unique.fdc_id, 1455408);
    assert_eq!(unique.data_type, "Branded");
    assert_eq!(unique.description, "WESSON Canola Oil 24 FL OZ");

    // second search is a phrase
    let mut results = service
        .v1_foods_search(&client, "Cheddar Cheese")
        .await
        .unwrap();
    let cheese = results.pop().unwrap();
    assert_eq!(cheese.description, "CHEDDAR CHEESE");
}

#[tokio::test]
#[ignore]
async fn v1_foods() {
    // get the service and a client
    let service = get_service();
    let client = reqwest::Client::new();

    // search one of each type of food
    let slice = [1455408, 173323, 1103005, 329370];
    let mut results = service.v1_foods(&client, &slice).await.unwrap();
    assert_eq!(results.len(), 4);

    // check the foundation
    let foundation = results.pop().unwrap();
    match foundation {
        FDCMeta::Other(meta) => {
            assert_eq!(meta.fdc_id, slice[3]);
            assert_eq!(meta.food_portions[0].id, 119685);
        }
        _ => {
            panic!("Should have been a foundation food!");
        }
    };

    // check the survey
    let survey = results.pop().unwrap();
    match survey {
        FDCMeta::Other(meta) => {
            assert_eq!(meta.fdc_id, slice[2]);
            assert_eq!(meta.food_attributes[0].id, 998724);
            assert_eq!(meta.food_portions[0].id, 239434);
        }
        _ => {
            panic!("Should have been a survey food!");
        }
    };

    // check the sr legacy
    let legacy = results.pop().unwrap();
    match legacy {
        FDCMeta::Other(meta) => {
            assert_eq!(meta.fdc_id, slice[1]);
            assert!(meta.food_attributes.is_empty());
            assert_eq!(meta.food_portions[0].id, 92296);
        }
        _ => {
            panic!("Should have been an sr legacy food!");
        }
    };

    // check the branded food
    let branded = results.pop().unwrap();
    match branded {
        FDCMeta::Branded(meta) => {
            assert_eq!(meta.fdc_id, slice[0]);
            assert_eq!(meta.label_nutrients.map(|ns| ns.fat.value), Some(13.9995));
        }
        _ => {
            panic!("Should have been a branded food!");
        }
    };
}
