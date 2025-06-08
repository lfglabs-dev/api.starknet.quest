pub mod tests {
    use reqwest::StatusCode;

    #[tokio::test]
    pub async fn test_fail_without_address() {
        let endpoint = format!("http://0.0.0.0:8080/defi/rewards");
        let client = reqwest::Client::new();
        let response = client.get(endpoint).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    pub async fn test_fail_with_invalid_address_format() {
        let address = "0x03fbb5d22e1393e47ff967u88urui3u4iyr3ui4r90sduw0943jowefwruwerowu";
        let endpoint = format!("http://0.0.0.0:8080/defi/rewards?addr={}", address);
        let client = reqwest::Client::new();
        let response = client.get(endpoint).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    pub async fn test_ok_with_valid_address_format() {
        let address = "0x03fbb5d22e1393e47ff9678d12748885f176d8ce96051f72819cd2a6fa062589";
        let endpoint = format!("http://0.0.0.0:8080/defi/rewards?addr={}", address);
        let client = reqwest::Client::new();
        let response = client.get(endpoint).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    pub async fn test_ok_with_address_without_callback() {
        let address = "0x05f1f8de723d8117daa26ec24320d0eacabc53a3d642acb0880846486e73283a";
        let endpoint = format!("http://0.0.0.0:8080/defi/rewards?addr={}", address);
        let client = reqwest::Client::new();
        let response = client.get(endpoint).send().await.unwrap();

        // Should return OK status even when no rewards are available
        assert_eq!(response.status(), StatusCode::OK);

        // Response should be an empty array
        let rewards: Vec<serde_json::Value> = response.json().await.unwrap();
        assert!(
            rewards.is_empty(),
            "Expected empty rewards array for address without callback data"
        );
    }

    #[tokio::test]
    pub async fn test_update_quest_with_banner() {
        use crate::models::Banner;
        use mongodb::bson::{doc, to_bson, from_bson};
    
        let banner = Banner {
            tag: "Test tag".to_string(),
            title: "Test title".to_string(),
            description: "Test description".to_string(),
            cta: "Test cta".to_string(),
            href: "https://test.com/event".to_string(),
            image: "https://test.com/image.png".to_string(),
        };
    
        //Serialization and deserialization
        let bson_banner = to_bson(&banner).unwrap();
        let deserialized_banner: Banner = from_bson(bson_banner.clone()).unwrap();
        assert_eq!(banner.tag, deserialized_banner.tag);
        assert_eq!(banner.title, deserialized_banner.title);
    
        //Inserting into the update document
        let mut update_doc = doc! {};
        update_doc.insert("banner", bson_banner.clone());
        assert_eq!(update_doc.get("banner").unwrap(), &bson_banner);
    } 



    // Unit tests for the CSV creation logic
    #[tokio::test]
    pub async fn test_csv_creation_logic() {
        use crate::endpoints::admin::quest_boost::get_boost_winners_csv::create_csv_response;
        
        // Test with winners
        let winners = Some(vec![
            "0x1234567890abcdef".to_string(),
            "0xfedcba0987654321".to_string(),
        ]);
        let boost_id = 123;
        
        let result = create_csv_response(&winners, boost_id).unwrap();
        
        let expected = "boost_id,winner_address,position\n\
                       123,0x1234567890abcdef,1\n\
                       123,0xfedcba0987654321,2\n";
        
        assert_eq!(result, expected);
    }

    #[tokio::test]
    pub async fn test_csv_creation_no_winners() {
        use crate::endpoints::admin::quest_boost::get_boost_winners_csv::create_csv_response;
        
        // Test with no winners
        let winners = None;
        let boost_id = 456;
        
        let result = create_csv_response(&winners, boost_id).unwrap();
        
        let expected = "boost_id,winner_address,position\n";
        
        assert_eq!(result, expected);
    }

    #[tokio::test]
    pub async fn test_csv_creation_empty_winners() {
        use crate::endpoints::admin::quest_boost::get_boost_winners_csv::create_csv_response;
        
        // Test with empty winners vector
        let winners = Some(vec![]);
        let boost_id = 789;
        
        let result = create_csv_response(&winners, boost_id).unwrap();
        
        let expected = "boost_id,winner_address,position\n";
        
        assert_eq!(result, expected);
    }   
   

}
