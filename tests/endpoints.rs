pub mod tests {
    use reqwest::StatusCode;
    use std::env::var;

    pub fn setup() {
        dotenvy::dotenv().ok();
    }

    #[tokio::test]
    pub async fn test_fail_without_address() {
        setup();
        let endpoint = format!("{}/defi/rewards", var("HOST").unwrap());
        let client = reqwest::Client::new();
        let response = client.get(endpoint).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    pub async fn test_fail_with_invalid_address_format() {
        setup();
        let address = "0x03fbb5d22e1393e47ff967u88urui3u4iyr3ui4r90sduw0943jowefwruwerowu";
        let endpoint = format!("{}/defi/rewards?addr={}", var("HOST").unwrap(), address);
        let client = reqwest::Client::new();
        let response = client.get(endpoint).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    pub async fn test_ok_with_valid_address_format() {
        setup();
        let address = "0x03fbb5d22e1393e47ff9678d12748885f176d8ce96051f72819cd2a6fa062589";
        let endpoint = format!("{}/defi/rewards?addr={}", var("HOST").unwrap(), address);
        let client = reqwest::Client::new();
        let response = client.get(endpoint).send().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
