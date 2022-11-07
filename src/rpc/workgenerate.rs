use serde::{Deserialize, Serialize};

use super::process::post_node;

#[derive(Serialize, Deserialize, Debug)]
struct WorkRequest {
    action: String,
    hash: String,
    difficulty: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct WorkResponse {
    work: String,
    difficulty: String,
    multiplier: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ValidateRequest {
    action: String,
    difficulty: String,
    work: String,
    hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ValidateResponse {
    valid: String,
}

pub fn get_server_work(
    hash: &[u8; 32],
    difficulty: &str,
    server_url: &str,
) -> Result<String, String> {
    let body = serde_json::to_string(&WorkRequest {
        action: String::from("work_generate"),
        hash: hex::encode(hash),
        difficulty: String::from(difficulty),
    })
    .unwrap();
    let response = post_node(body, server_url, 30)?;
    let work_response: Result<WorkResponse, _> = serde_json::from_str(&response);
    match work_response {
        Ok(work_response) => Ok(work_response.work),
        Err(e) => {
            //eprintln!("Error getting account info: {}", e);
            Err(e.to_string())
        }
    }
}

pub fn test_work_server(server_url: &str) -> Result<String, String> {
    let body = serde_json::to_string(&ValidateRequest {
        action: String::from("work_validate"),
        difficulty: String::from("FFFFFFF800000000"),
        work: String::from("e4fa6e9a3ff5227d"),
        hash: String::from("36043971B045ECC7090130F445287921B24369AEDC099A6504DBA92868A28BB4"),
    })
    .unwrap();
    let response = post_node(body, server_url, 5)?;
    let validate_response: Result<ValidateResponse, _> = serde_json::from_str(&response);
    match validate_response {
        Ok(validate_response) => {
            if validate_response.valid == "1" {
                Ok(String::from("Success."))
            } else {
                Err(String::from(
                    "the response from the work server was incorrect.",
                ))
            }
        }
        Err(e) => {
            //eprintln!("Error testing work server: {}", e);
            Err(e.to_string())
        }
    }
}
