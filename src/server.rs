use std::io::Empty;
use std::ops::{Deref, Div};
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;
use std::{default, env, fs, io};
use std::{thread, time::Duration};

use std::os::unix::fs::symlink;

use chrono::format;
use clap::{error, Error};
use log::{error, info, trace};
use log4rs::encode::json;
use redis::{Commands, Connection, RedisError, Script, Value};
use serde::{Deserialize, Serialize};
use serde_json::{json, value};

use ansi_term::Colour::{Black, Blue, Cyan, Green, Purple, Red, White, Yellow};
use ansi_term::{Color, Style};
use regex::Regex;

/* WORKER_TERMINAL_COLORS

   @dev Colors used to distinguish different threads
   @author wizdaydream@gmail.com
*/
const WORKER_TERMINAL_COLORS: [Color; 20] = [
    Color::RGB(238, 63, 77),
    Color::RGB(43, 18, 22),
    Color::RGB(233, 184, 195),
    Color::RGB(167, 168, 189),
    Color::RGB(46, 49, 124),
    Color::RGB(23, 114, 180),
    Color::RGB(97, 113, 114),
    Color::RGB(85, 187, 138),
    Color::RGB(208, 222, 170),
    Color::RGB(249, 211, 103),
    Color::RGB(237, 51, 51),
    Color::RGB(244, 62, 6),
    Color::RGB(232, 180, 154),
    Color::RGB(102, 70, 42),
    Color::RGB(247, 193, 115),
    Color::RGB(57, 55, 51),
    Color::RGB(242, 230, 206),
    Color::RGB(88, 71, 23),
    Color::RGB(252, 210, 23),
    Color::RGB(91, 174, 35),
];

macro_rules! color_log {
  ($color:expr, info, $($msg:tt)*) => {
    info!("{}", $color.paint(format!($($msg)*)));
  };
  ($color:expr, error, $($msg:tt)*) => {
    error!("{}", $color.paint(format!($($msg)*)));
  };
  ($color:expr, trace, $($msg:tt)*) => {
    trace!("{}", $color.paint(format!($($msg)*)));
  };
}

#[derive(Debug, Serialize, Deserialize)]
enum ErrorCode {
    RedisConnectErr(String),
    DataError(String),
    DataNotString(String),
    DataNotJson(String),
    NoNewMessage,
    ReadDirErr(String),
    ForgeBuildFailure(String),
    ForgeCompileFailure(String),
    ForgeTestFailure(String),
    ResultJsonReadFailure(String),
    EmptyFile,
}

impl ErrorCode {
    fn get_err_msg(&self) -> String {
        match self {
            ErrorCode::RedisConnectErr(err) => {
                return String::from(format!("Redis connect exception: {}", err));
            }
            ErrorCode::DataError(err) => {
                return String::from(format!("Data err {}", err));
            }
            ErrorCode::DataNotJson(err) => {
                return String::from(format!("Data not json {}", err));
            }
            ErrorCode::DataNotString(data) => {
                return String::from(format!("Data not string type, {}", data));
            }
            ErrorCode::NoNewMessage => {
                return String::from("No new message.");
            }
            ErrorCode::ReadDirErr(data) => {
                return String::from(format!("Data not string type, {}", data));
            }
            ErrorCode::ForgeBuildFailure(data) => {
                return String::from(format!("{}", data));
            }
            ErrorCode::ForgeTestFailure(data) => {
                return String::from(format!("{}", data));
            }
            ErrorCode::ForgeCompileFailure(data) => {
                return String::from(format!("{}", data));
            }
            ErrorCode::ResultJsonReadFailure(data) => {
                return String::from(format!("{}", data));
            }
            ErrorCode::EmptyFile => {
                return String::from("Empty file");
            }
            _ => {
                return String::new();
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MyError {
    err_msg: String,
    err_code: i8,
}

#[derive(Debug, Serialize, Deserialize)]
struct PathWithContent {
    #[serde(rename = "path")]
    path: String,

    #[serde(rename = "content")]
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JobMessage {
    #[serde(rename = "questionNo")]
    question_no: String,

    #[serde(rename = "solcVersion")]
    solc_version: String,

    #[serde(rename = "judgeJobId")]
    judge_job_id: String,

    #[serde(rename = "jobKey")]
    job_key: String,

    #[serde(rename = "pathWithContent")]
    path_with_content: Vec<PathWithContent>,
}

impl MyError {
    fn new(_err: &str, _err_code: i8) -> MyError {
        MyError {
            err_msg: _err.to_string(),
            err_code: _err_code,
        }
    }
}

fn get_value_str(v: Value) -> Result<String, ErrorCode> {
    match v {
        Value::Data(bytes) => {
            let s = String::from_utf8_lossy(&bytes).into_owned();
            return Ok(s);
        }
        _ => {
            return Err(ErrorCode::DataNotString(format!("{:?}", v)));
        }
    }
}

fn create_files_as_job_message(
    job: &JobMessage,
    worker_num: i8,
    worker_dir: &str,
) -> Result<(), ErrorCode> {
    let questionNo = &job.question_no;
    let base_path = Path::new(worker_dir)
        .join(format!("{:02}", worker_num))
        .join(&job.question_no);

    let output_path = base_path.join("output");
    if job.path_with_content.len() == 0 {
        // let mut json = json!({});
        // json["info"] = json!("Result not json");
        // json["code"] = json!(2);
        // json["msg"] = json!("No files");
        // let _ = fs::write(output_path.join("output.json"), json.to_string());
        return Err(ErrorCode::EmptyFile);
    }
    for j in &job.path_with_content {
        let path = Path::new(&worker_dir)
            .join(format!("{:02}", worker_num))
            .join(&job.question_no)
            .join(&j.path);
        if !path.is_file() {
            let rIndex = path.as_os_str().to_str().unwrap().rfind('/').unwrap();
            let path_sub_str = &path.as_os_str().to_str().unwrap()[0..rIndex];
            if Path::new(path_sub_str).exists() {
            } else {
                fs::create_dir_all(path_sub_str).unwrap();
            }
            fs::write(path.clone(), &j.content).unwrap();
        }
        // info!("{}", color.paint(format!("{}", path.as_os_str().to_str().unwrap())));
    }
    Ok(())
}

fn get_job_message_by_redis_value(rv: redis::Value) -> Result<JobMessage, ErrorCode> {
    match rv {
        Value::Bulk(data) => {
            let list_name = data.get(0).unwrap();
            let data_str = data.get(1).unwrap();

            let data = get_value_str(data_str.clone());
            if data.is_err() {
                let err =
                    ErrorCode::DataError(format!("Data err({})", data.unwrap_err().get_err_msg()));
                return Err(err);
            } else {
            }

            let data = data.unwrap();
            let jobTransferRes: Result<JobMessage, serde_json::Error> =
                serde_json::from_str(data.as_str());
            if jobTransferRes.is_err() {
                let err = ErrorCode::DataNotJson(format!(
                    "Data err({})",
                    jobTransferRes.unwrap_err().to_string()
                ));
                return Err(err);
            } else {
            }

            let job = jobTransferRes.unwrap();
            return Ok(job);
        }
        Value::Nil => {
            return Err(ErrorCode::NoNewMessage);
        }
        _ => {
            return Err(ErrorCode::DataError(format!("Data err(Unknown)")));
        }
    }
}

fn worker_thread(
    num: i8,
    redis_host: &str,
    redis_prefix: &str,
    worker_dir: &str,
    redis_list_name: &str,
) -> Result<(), ErrorCode> {
    let color = WORKER_TERMINAL_COLORS[num as usize];

    let connection_str = format!("redis://{}", redis_host);

    let redis_client_result = redis::Client::open(connection_str);

    let base_path = Path::new(worker_dir).join(format!("{:02}", num).as_str());

    if redis_client_result.is_err() {
        let redis_err = redis_client_result.as_ref().unwrap_err();
        let err = ErrorCode::RedisConnectErr(redis_err.to_string());
        color_log!(color, error, "[Thread {}:] {}", num, err.get_err_msg());
        return Err(err);
    } else {
    }

    let redis_client = redis_client_result.unwrap();

    let conn_res = redis_client.get_connection();
    if conn_res.is_err() {
        let redis_err = conn_res.as_ref().err().unwrap();
        let err = ErrorCode::RedisConnectErr(redis_err.to_string());
        color_log!(color, error, "[Thread {}:] {}", num, err.get_err_msg());
        return Err(err);
    }

    let mut conn = conn_res.unwrap();

    loop {
        // sleep 20ms
        thread::sleep(Duration::from_millis(20));
        let x: Result<Value, RedisError> = conn.blpop(&redis_list_name, 1);

        if x.is_err() {
            // error!("{}", x.unwrap_err().to_string());
            continue;
        } else {
        }

        let job_res = get_job_message_by_redis_value(x.unwrap());
        if job_res.is_err() {
            let err = job_res.unwrap_err();
            match err {
                ErrorCode::NoNewMessage => {
                    color_log!(color, trace, "[Thread {}:] {}", num, err.get_err_msg());
                }
                _ => {
                    color_log!(color, error, "[Thread {}:] {}", num, err.get_err_msg());
                }
            }
            continue;
        }
        let job = job_res.unwrap();
        let start_time = SystemTime::now();
        color_log!(color, trace, "[Thread {}:] Received job: {:?}", num, &job);
        color_log!(
            color,
            trace,
            "[Thread {}:] Received job: {}, start creating files...",
            num,
            job.question_no
        );
        //  Create files as the path
        let res = create_files_as_job_message(&job, num, worker_dir);
        if let Err(ErrorCode::EmptyFile) = res {
          color_log!(
              color,
              error,
              "[Thread {}:] No files created, check {}",
              num,
              &job.question_no
          );
        }

        // Start forge build

        color_log!(
            color,
            trace,
            "[Thread {}:] Creating files complete: {}, start testing...",
            num,
            &job.question_no
        );
        let _ = collect_output_from_test_scripts(
            Path::new("tmp/worker")
                .join(format!("{:02}", num))
                .join(&job.question_no)
                .as_path(),
        );

        let forge_test_res = run_forge_test(&job, num);

        let output_path = Path::new("tmp/worker")
            .join(format!("{:02}", num))
            .join(&job.question_no)
            .join("output")
            .join("output.json");
        if forge_test_res.is_err() {
            match forge_test_res.err().unwrap() {
                ErrorCode::ForgeBuildFailure(data) => {
                    color_log!(
                        color,
                        error,
                        "[Thread {}:] Forge bulild run failed, result saved in {}",
                        num,
                        output_path.to_str().unwrap()
                    );
                }
                ErrorCode::ForgeTestFailure(data) => {
                    color_log!(
                        color,
                        error,
                        "[Thread {}:] Forge test run failed, result saved in {}",
                        num,
                        output_path.to_str().unwrap()
                    );
                }
                ErrorCode::ForgeCompileFailure(data) => {
                    color_log!(
                        color,
                        error,
                        "[Thread {}:] Forge compile failed, result saved in {}",
                        num,
                        output_path.to_str().unwrap()
                    );
                }
                ErrorCode::EmptyFile => {
                    color_log!(
                        color,
                        error,
                        "[Thread {}:] Forge test failed, result saved in {}",
                        num,
                        output_path.to_str().unwrap()
                    );
                }
                _ => {}
            }
        } else {
            let cost_time = SystemTime::now().duration_since(start_time).unwrap();
            color_log!(
                color,
                info,
                "[Thread {}:] Test complete in {:.2}s, result saved in {}. Start cleanning...",
                num,
                cost_time.as_millis() as f64 / 1000.0,
                output_path.to_str().unwrap()
            );
        }

        let request_key = format!("{}:{}", &job.job_key, "request");
        color_log!(color, info, "[Thread {}:] Output path {}", num, &output_path.as_os_str().to_str().unwrap());
        let request_v: Result<redis::Value, RedisError> = conn.get(&request_key);
        if request_v.is_err() {
            color_log!(
                color,
                error,
                "[Thread {}:] No such key {}",
                num,
                &request_key
            );
        } else {
            let response_key = format!("{}:{}", &job.job_key, "response");
            // color_log!(color, error, "{}", output_path.as_os_str().to_str().unwrap());
            let _ = write_response_if_request_exist(
                &request_key,
                &response_key,
                &fs::read_to_string(output_path).unwrap(),
                &mut conn,
            );
        }
        // let _ = clean_contracts_and_test_dir(&job, num);
    }
    Ok(())
}

fn write_response_if_request_exist(
    request_key: &str,
    response_key: &str,
    response: &str,
    conn: &mut redis::Connection,
) -> Result<(), ErrorCode> {
    let script = Script::new(
        "
    if redis.call('exists', KEYS[1]) == 1 then
        redis.call('del', KEYS[1])
        redis.call('set', KEYS[2], ARGV[1])
        return 1
    else
        return 0
    end
  ",
    );

    let res = script
        .key(request_key)
        .key(response_key)
        .arg(response)
        .invoke::<i32>(conn);
    if res.is_err() {
        println!("{}", res.unwrap_err());
    }
    Ok(())
}

fn clean_contracts_and_test_dir(job: &JobMessage, worker_num: i8) -> Result<(), ErrorCode> {
    let base_path = Path::new("tmp/worker").join(format!("{:02}", worker_num));

    let _ = fs::remove_dir_all(base_path.join("contracts"));
    let _ = fs::remove_dir_all(base_path.join("test"));
    let _ = fs::remove_dir_all(base_path.join("output"));
    Ok(())
}

fn run_forge_test(job: &JobMessage, worker_num: i8) -> Result<String, ErrorCode> {
    let base_path = Path::new("tmp/worker")
        .join(format!("{:02}", worker_num))
        .join(&job.question_no);
    let cache_path = base_path.join("cache");

    let contracts_path = base_path.join("contracts");
    let test_path = base_path.join("test");
    let output_path = base_path.join("output");

    let out_path = base_path.join("out");
    let out_version_path = out_path.join(job.solc_version.as_str());

    // let _ = fs::create_dir(&out_version_path);
    let _ = fs::create_dir(&cache_path);
    let _ = fs::create_dir(&out_path);
    let _ = fs::create_dir(&output_path);

    // First initialize the cache json
    if cache_path.join("solidity-files-cache.json").exists() {
    } else {
        let _ = fs::copy(
            "cache/example-cache.json",
            cache_path.join("solidity-files-cache.json"),
        );
        let mut example_cache: serde_json::Value = serde_json::from_str(
            fs::read_to_string("cache/example-cache.json")
                .unwrap()
                .as_str(),
        )
        .unwrap();
        /*

         "artifacts": "tmp_out",
         "build_infos": "tmp_out/build-info",
         "sources": "[fixme]",
         "tests": "test",
         "scripts": "script",
         "libraries": [
           "lib",
           "node_modules"
         ]
        */
        example_cache["paths"]["artifacts"] = json!(out_path.as_os_str().to_str());
        example_cache["paths"]["build_infos"] =
            json!(out_path.join("build-info").as_os_str().to_str());
        example_cache["paths"]["sources"] = json!(base_path.as_os_str().to_str());
        example_cache["paths"]["tests"] = json!("test");

        let _ = fs::write(
            cache_path.join("solidity-files-cache.json"),
            example_cache.to_string(),
        );

        // info!("{} {}", Path::new("out").join(job.solc_version.as_str()).as_os_str().to_str().unwrap(), out_version_path.to_str().unwrap());
        let src_path_str = env::current_dir()
            .unwrap()
            .to_path_buf()
            .join(Path::new("out"))
            .join(job.solc_version.as_str())
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
        let dst_path_str = env::current_dir()
            .unwrap()
            .to_path_buf()
            .join(out_version_path)
            .to_string_lossy()
            .into_owned();
        let _ = symlink(&src_path_str, &dst_path_str);
    }

    // error!("{}", res.unwrap_err().to_string());

    let output_path = base_path.join("output");
    let contracts_path = base_path.join("contracts");
    let test_path = base_path.join("test");

    let res = Command::new("forge")
        .args([
            "build",
            "--contracts",
            base_path.as_os_str().to_str().unwrap(),
            "--cache-path",
            cache_path.as_os_str().to_str().unwrap(),
            "--out",
            out_path.as_os_str().to_str().unwrap(),
            "--use",
            &job.solc_version,
        ])
        .output();

    if res.is_err() {
        // error!("{}", res.as_ref().unwrap_err().to_string());
        return Err(ErrorCode::ForgeBuildFailure(res.unwrap_err().to_string()));
    }
    let stderr = String::from_utf8(res.unwrap().stderr).unwrap();
    if stderr.len() > 0 {
        // Compile failure
        let re = Regex::new(r"\x1b\[[\d;]+m").unwrap();
        let result = re.replace_all(&stderr.as_str(), "").to_string();

        let mut json = json!({});
        json["info"] = json!("Compile failed");
        json["code"] = json!(1);
        json["msg"] = json!(result.as_str());
        let _ = fs::write(output_path.join("output.json"), json.to_string());
        return Err(ErrorCode::ForgeCompileFailure(result));
    }

    let res = Command::new("forge")
        .args([
            "test",
            "--contracts",
            base_path.as_os_str().to_str().unwrap(),
            "--cache-path",
            cache_path.as_os_str().to_str().unwrap(),
            "--out",
            out_path.as_os_str().to_str().unwrap(),
            "--json",
            "--use",
            &job.solc_version,
            "--offline",
            "--allow-failure",
        ])
        .output();

    if res.is_err() {
        // error!("{}", res.as_ref().unwrap_err().to_string());
        return Err(ErrorCode::ForgeTestFailure(res.unwrap_err().to_string()));
    }

    let stdout = String::from_utf8(res.unwrap().stdout).unwrap();

    let test_out_json_res: Result<serde_json::Value, serde_json::Error> =
        serde_json::from_str(&stdout);
    if test_out_json_res.is_err() {
        color_log!(
            WORKER_TERMINAL_COLORS[worker_num as usize],
            error,
            "Test out is not json."
        );


        let mut json = json!({});
        json["info"] = json!("Result not json");
        json["code"] = json!(2);
        json["msg"] = json!("No files");


        let write_res = fs::write(output_path.join("output.json"), json.to_string());
        if write_res.is_err(){
        color_log!(
          WORKER_TERMINAL_COLORS[worker_num as usize],
          error,
          "Write file error {}",
          write_res.unwrap_err().to_string()
      );
        }
        return Err(ErrorCode::EmptyFile);
    }
    let test_out_json: serde_json::Value = test_out_json_res.unwrap();

    let output_res: Result<serde_json::Value, serde_json::Error> = serde_json::from_str(
        fs::read_to_string(output_path.join("output.json"))
            .unwrap()
            .as_str(),
    );
    if output_res.is_err() {
        // error!("{}", raw_output_res.as_ref().unwrap_err().to_string());
        return Err(ErrorCode::ResultJsonReadFailure(
            output_res.unwrap_err().to_string(),
        ));
    }
    let mut output = output_res.unwrap();

    let mut total_score = 0;
    let mut get_score = 0;

    for (k, v) in test_out_json.as_object().unwrap() {
        let obj = v.as_object().unwrap();
        let test_results = obj.get("test_results").unwrap();
        for q in output["questions"].as_array_mut().unwrap() {
            let case = &test_results[format!("{}{}", q["Func"].as_str().unwrap(), "()")];

            // info!("{:#}", case);

            q["Score"] = json!(q["Score"].as_str().unwrap_or("0").parse::<i64>().unwrap());

            total_score += q["Score"].as_i64().unwrap();

            if case["status"].as_str().is_none() {
                // This question is not collected in the reg match phase, pass
                continue;
            }

            if case["status"].as_str().unwrap() == "Failure" {
                q["Passed"] = json!(false);
            } else {
                q["Passed"] = json!(true);
                get_score += q["Score"].as_i64().unwrap();
            }
        }
    }

    output["total_score"] = json!(total_score);
    output["get_score"] = json!(get_score);
    output["info"] = json!("Complete");
    output["code"] = json!(0);
    output["msg"] = json!("Complete");

    let _ = fs::write(output_path.join("output.json"), output.to_string());

    Ok((output.to_string()))
}

fn clean_project(p: &str) {
    let _ = fs::remove_dir_all(Path::new(p));
}

fn collect_output_from_test_scripts(basepath: &Path) -> Result<(), ErrorCode> {
    let test_dir = Path::new(basepath).join("test");
    let contract_dir = Path::new(basepath).join("contracts");
    let output = Path::new(basepath).join("output");

    let _ = fs::remove_dir_all(output.clone());
    let res = fs::create_dir_all(output.clone());


    let test_dir_entries_res = fs::read_dir(test_dir);
    if test_dir_entries_res.is_err() {
        return Err(ErrorCode::ReadDirErr(format!(
            "Read directory error({})",
            test_dir_entries_res.unwrap_err().to_string()
        )));
    }
    let test_dir_entries = test_dir_entries_res.unwrap();

    let mut raw_infos: Vec<serde_json::Value> = vec![];

    for entry in test_dir_entries {
        let entry_dir = entry.unwrap();
        let path = entry_dir.path();
        let file_content = fs::read(&path).unwrap();
        let file_content = String::from_utf8_lossy(&file_content).into_owned();
        // println!("{}", file_content);

        let re = Regex::new(r"\/\*[^@]*(@[^\@\/]*)+\*\/\s*function\s(test_[^\(]+)").unwrap();

        let res: Vec<&str> = re.find_iter(&file_content).map(|m| m.as_str()).collect();

        for s in res {
            let attributes_re = Regex::new(r"@.*[\s]*:[\s]*[^\n]*").unwrap();
            let res: Vec<&str> = attributes_re.find_iter(&s).map(|m| m.as_str()).collect();
            let mut res_json = json!({});
            for attr in res {
                let split: Vec<&str> = attr[1..].split(":").collect();
                let attribute_name = split[0].trim();
                let attribute_value = split[1].trim();
                res_json[attribute_name] = json!(attribute_value);
            }
            let function_name_re = Regex::new(r"function\s(test[^\(]*)").unwrap();
            let mut function_name: Option<String> = None;
            for (_, [function_name_captured]) in
                function_name_re.captures_iter(&s).map(|s| s.extract())
            {
                function_name = Some(function_name_captured.to_string());
            }
            res_json["Func"] = json!(function_name.unwrap());
            res_json["Passed"] = json!(false);
            raw_infos.push(res_json);
        }
    }

    let mut test_res_json = json!({});
    test_res_json["questions"] = json!(raw_infos);

    let _ = fs::write(
        output.join(format!("output.json")),
        test_res_json.to_string(),
    );

    Ok(())
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> io::Result<()> {
    fs::create_dir_all(&dst)?; // 确保目标目录存在
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            // 如果是一个目录，递归地复制这个目录
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            // 如果是一个文件，直接复制这个文件
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

fn init_cache_file() -> Result<(), ErrorCode> {
    if Path::new("cache/example-cache.json").exists() {
        return Ok(());
    }

    let version_vec = vec!["0.8.20", "0.8.21", "0.8.22", "0.8.23"];

    // Fistly, rebuild the out directory where the cache will be stored
    let _ = fs::remove_dir_all("out");
    let _ = fs::create_dir("out");
    let _ = fs::remove_dir_all("cache");

    let _ = Command::new("forge").arg("clean").output();

    let init_version = version_vec[0];
    let _ = Command::new("forge")
        .args([
            "build",
            "--contracts",
            "lib/forge-std/src",
            "--out",
            "tmp_out",
            "--use",
            init_version,
        ])
        .output();

    let mut example_cache: serde_json::Value =
        serde_json::from_str(&fs::read_to_string("cache/solidity-files-cache.json").unwrap())
            .unwrap();
    example_cache["paths"]["sources"] = json!("fixme");

    // for &v in & version_vec[1..]{
    let lib_files_iter = example_cache["files"]
        .as_object()
        .unwrap()
        .keys()
        .filter(|&s| s.starts_with("lib"))
        .cloned()
        .collect::<Vec<String>>();
    for f in lib_files_iter {
        for (file, compile_info) in example_cache["files"][f]["artifacts"]
            .as_object_mut()
            .unwrap()
        {
            for (compiler, loc) in compile_info.as_object_mut().unwrap() {
                let loc_str = loc.as_str().unwrap();
                *loc = json!(format!("{}/{}", init_version, loc_str));
            }
        }
    }
    // }

    let _ = copy_dir_all("tmp_out", Path::new("out").join(init_version));

    // const initObj = JSON.parse(String(fs.readFileSync("cache/solidity-files-cache.json")));

    // initObj.paths.sources = "[fixme]";

    // for( const fName of Object.keys(initObj.files)){
    //   const artifacts = initObj.files[fName].artifacts;
    //   for(const cName of Object.keys(artifacts)){
    //     for(const [compiler, location] of Object.entries(artifacts[cName])){
    //       initObj.files[fName].artifacts[cName][compiler] = version + "/" + location;
    //     }
    //   }
    // }

    // execSync("cp -r tmp_out" + " out/" + version);

    for &v in &version_vec[1..] {
        let _ = Command::new("forge")
            .args([
                "build",
                "--contracts",
                "lib/forge-std/src",
                "--out",
                "tmp_out",
                "--use",
                v,
            ])
            .output();
        let mut v_cache: serde_json::Value =
            serde_json::from_str(&fs::read_to_string("cache/solidity-files-cache.json").unwrap())
                .unwrap();

        for file in v_cache["files"].as_object().unwrap().keys() {
            let artifacts = v_cache["files"][file]["artifacts"].as_object().unwrap();
            for a in artifacts.keys() {
                for (compiler, loc) in artifacts.get(a).unwrap().as_object().unwrap() {
                    // for(compiler, loc) in compile_info.as_object().unwrap(){
                    example_cache["files"][file]["artifacts"][a][compiler] =
                        json!(format!("{}/{}", v, loc.clone().take().as_str().unwrap()));
                    // }
                }
            }
        }
        let _ = copy_dir_all("tmp_out", Path::new("out").join(v));
    }

    let _ = fs::write(
        "cache/example-cache.json",
        serde_json::to_string_pretty(&example_cache).unwrap(),
    );
    let _ = fs::remove_file("cache/solidity-files-cache.json");
    Ok(())
}

pub fn start(
    thread_num: i32,
    redis_host: &str,
    redis_prefix: &str,
    worker_dir: &str,
    redis_list_name: &str,
) {
    log4rs::init_file("log4rs.yaml", Default::default()).unwrap();
    // print!("{:?}", res.unwrap_err());
    info!("Start cleanning..");
    clean_project(worker_dir);
    info!("Start make example cache and prepare artifacts...");
    let _ = init_cache_file();
    let mut handles = vec![];
    for i in 0..thread_num {
        info!("Starting thread {}", i);
        let redis_host_str = redis_host.to_string();
        let redis_prefix_str = redis_prefix.to_string();
        let worker_dir_str = worker_dir.to_string();
        let redis_list_name_str = redis_list_name.to_string();

        let handle = thread::spawn(move || {
            let res = worker_thread(
                i as i8,
                redis_host_str.as_str(),
                redis_prefix_str.as_str(),
                worker_dir_str.as_str(),
                redis_list_name_str.as_str(),
            );
            if res.is_err() {
                log::error!("{:?}", res.unwrap_err().get_err_msg());
            } else {
                print!("222");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
