use std::{
    fs::{self, DirEntry},
    io,
    path::{Path, PathBuf},
    time::SystemTime,
};

use clap::{error, Error};
use log::info;
use log4rs::encode::json;
use redis::{Commands, ConnectionLike, Script};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::{timeout, Duration};

use crate::server::start;

#[derive(Debug)]
pub enum ErrorCode {
    RedisInitErr(redis::RedisError),
    RedisConnectErr(redis::RedisError),
    DirectoryNotFound(String),
    ProcessTimeout,
    ProcessNotFinished,
}

impl ErrorCode {}

static REDIS_CONNECTION_STR: &'static str = "redis://127.0.0.1";

fn read_files_to_json(
    p: String,
    solc_version: String,
    question_no: String,
    job_id: String,
) -> Result<String, ErrorCode> {
    let base_path = Path::new(p.as_str());

    if !base_path.exists() {
        return Err(ErrorCode::DirectoryNotFound(p.clone()));
    }

    let mut json: Vec<serde_json::Value> = vec![];
    let _ = recursive_get_string(&PathBuf::from(p.as_str()), PathBuf::new(), &mut json);
    let mut send_obj = json!({});
    send_obj["questionNo"] = json!(question_no);
    send_obj["pathWithContent"] = serde_json::Value::Array(json);
    send_obj["judgeJobId"] = json!(job_id);
    send_obj["solcVersion"] = json!(solc_version);
    send_obj["jobKey"] = json!(format!("smc-open-foundry-judge:{}", question_no));

    Ok(send_obj.to_string())
}

pub fn request(
    p: String,
    solc_version: String,
    question_no: String,
    job_id: String,
    timeout: String,
) -> Result<(), ErrorCode> {
    let start_time = SystemTime::now();
    let get_conn_res = get_redis_conn();
    if get_conn_res.is_err() {
        // println!("Redis connection error");
        return Err(get_conn_res.err().unwrap());
    }
    let mut conn = get_conn_res.unwrap();

    let s = read_files_to_json(p, solc_version, question_no.clone(), job_id.clone());
    if s.is_err() {
        let mut json = json!({});
        json["info"] = json!("Failed");
        json["code"] = json!(-3);
        json["msg"] = json!("Path not exist");
        json["jobId"] = json!(job_id.as_str());

        print!("{:#}", json);
        return Err(s.unwrap_err());
    }

    let rt = tokio::runtime::Runtime::new().unwrap();

    let request_key = format!("smc-open-foundry-judge:{}:request", &question_no);
    let response_key = format!("smc-open-foundry-judge:{}:response", question_no);

    let res = rt.block_on(process_with_timeout(
        &Duration::from_secs(timeout.parse::<u64>().unwrap()),
        s.unwrap().as_str(),
        &mut conn,
        &request_key,
        &response_key,
    ));

    let mut json = json!({});
    if res.is_err() {
        let _ = conn.del::<&str, i32>(&request_key);
        json["info"] = json!("Timeout");
        json["code"] = json!(-2);
        json["msg"] = json!("Timeout");
        json["jobId"] = json!(job_id.as_str());
    } else {
        // println!("{}", res.as_ref().unwrap());
        json = serde_json::from_str(res.unwrap().as_str()).unwrap();
    }

    let cost_time = SystemTime::now().duration_since(start_time).unwrap();
    json["costTime"] = json!(format!("{:.2}s", cost_time.as_secs_f64()));
    print!("{:#}", json);
    Ok(())
}

async fn process_with_timeout(
    duration: &Duration,
    value: &str,
    conn: &mut redis::Connection,
    request_key: &str,
    response_key: &str,
) -> Result<String, ErrorCode> {
    let async_operation = async move {
        let _ = conn.set::<&str, &str, i32>(request_key, "");
        let _ = conn.lpush::<&str, &str, i32>("test", value);

        for _ in 0..80 {
            let res = write_response_if_request_exist(&request_key, &response_key, conn);
            if res.is_err() {
                // println!("Not finished yet");
            } else {
                let data = res.unwrap();
                if data.len() > 0 {
                    return Ok(data.to_string());
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err::<String, ErrorCode>(ErrorCode::ProcessTimeout)
    };

    match timeout(*duration, async_operation).await {
        Ok(result) => return Ok(result.unwrap()),
        Err(_) => return Err(ErrorCode::ProcessTimeout),
    }
}

async fn interact_with_host_service(
    key: String,
    value: String,
    mut conn: Box<redis::Connection>,
    request_key: &str,
    response_key: &str,
) {
    println!("AAAAA");
}

fn write_response_if_request_exist(
    request_key: &str,
    response_key: &str,
    conn: &mut redis::Connection,
) -> Result<String, ErrorCode> {
    let script = Script::new(
        r#"
    if redis.call("EXISTS", KEYS[2]) == 1 then
        local value = redis.call("GET", KEYS[2])
        redis.call("DEL", KEYS[1])
        redis.call("DEL", KEYS[2])
        return value
    else
        return nil
    end
  "#,
    );

    let res = script
        .key(request_key)
        .key(response_key)
        .invoke::<Option<String>>(conn);
    if res.is_err() {
        println!("{}", res.as_ref().unwrap_err());
    }

    let res = res.unwrap();
    match res {
        None => return Err(ErrorCode::ProcessNotFinished),
        Some(s) => return Ok(s),
    }
}

fn recursive_get_string(
    base_path: &PathBuf,
    relative_path: PathBuf,
    list: &mut Vec<serde_json::Value>,
) -> Result<(), ErrorCode> {
    let cur_path = &base_path.join(&relative_path);
    let entries = cur_path.read_dir().unwrap();
    for e in entries {
        let dir_entry = e.unwrap();

        if dir_entry.metadata().unwrap().is_dir() {
            let _ = recursive_get_string(
                base_path,
                relative_path.join(dir_entry.file_name().into_string().unwrap()),
                list,
            );
        } else {
            let mut json = json!({});
            json["path"] = json!(relative_path
                .join(dir_entry.file_name())
                .to_str()
                .unwrap()
                .to_string());
            json["content"] = json!(fs::read_to_string(dir_entry.path()).unwrap());

            list.push(json);
        }
    }
    Ok(())
}

fn get_redis_conn() -> Result<Box<redis::Connection>, ErrorCode> {
    let redis_client_result = redis::Client::open(REDIS_CONNECTION_STR);

    if redis_client_result.is_err() {
        return Err(ErrorCode::RedisInitErr(redis_client_result.unwrap_err()));
    }

    let conn = redis_client_result.unwrap().get_connection();
    if conn.is_err() {
        return Err(ErrorCode::RedisConnectErr(conn.err().unwrap()));
    }

    Ok(Box::new(conn.unwrap()))
}
