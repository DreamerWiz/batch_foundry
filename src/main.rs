use std::{borrow::Borrow, env, process::exit};

use chrono::{Local, TimeZone};
use clap::{parser::ValueSource, Arg, ArgMatches, Command};
use log::trace;

mod client;
mod server;
mod types;

const COMMAND_NAME: &str = "test";
const VERSION: &str = "1.0";
const AUTHOR: &str = "Wiz Lee wizdaydream@gmail.com";
const ABOUT: &str = "Developer's tool for urlencode and time format!";

fn main() {
    let matches = Command::new(COMMAND_NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("server")
                .about("Start the server")
                .arg(
                    Arg::new("thread-num")
                        .short('t')
                        .long("thread-num")
                        .default_value("5")
                        .help("How many threads do you want(env: THREAD_NUM)"),
                )
                .arg(
                    Arg::new("redis-host")
                        .short('r')
                        .long("redis-host")
                        .default_value("redis://127.0.0.1/1")
                        .help("Redis server(env: REDIS_HOST)"),
                )
                .arg(
                    Arg::new("redis-prefix")
                        .short('p')
                        .long("redis-prefix")
                        .default_value("smc-open-solidity-judge")
                        .help("Prefix of the redis operations(env: REDIS_PREFIX)"),
                )
                .arg(
                    Arg::new("worker-dir")
                        .short('d')
                        .long("worker-dir")
                        .default_value("tmp/worker")
                        .help(
                            "The default work directory of the foundry env(env: REDIS_WORKER_DIR)",
                        ),
                )
                .arg(
                    Arg::new("redis-list-name")
                        .short('n')
                        .long("redis-list-name")
                        .default_value("test")
                        .help("Redis list name(env: REDIS_LIST_NAME)"),
                ),
        )
        .subcommand(
            Command::new("client")
                .about("Send a request to the redis and receive result")
                .arg(
                    Arg::new("directory")
                        .short('d')
                        .long("directory")
                        .default_value("usercode")
                        .help("Root dir of the files (env: DIRECTORY)"),
                )
                .arg(
                    Arg::new("question-no")
                        .short('n')
                        .long("question-no")
                        .help("Questino number of the input"),
                )
                .arg(
                    Arg::new("solc-version")
                        .short('v')
                        .long("solc-version")
                        .default_value("0.8.20")
                        .help("Solc version selected (env: SOLC_VERSION)"),
                )
                .arg(
                    Arg::new("job-id")
                        .short('j')
                        .long("job-id")
                        .help("Job id offered (env: JOB_ID)"),
                )
                .arg(
                    Arg::new("timeout")
                        .short('t')
                        .long("timeout")
                        .default_value("5")
                        .help("Timeout in secs (env: TIMEOUT)"),
                )
                .arg(
                    Arg::new("connection-str")
                        .short('c')
                        .long("connection-str")
                        .default_value("redis://127.0.0.1")
                        .help("Redis connection str"),
                ),
        )
        .subcommand(Command::new("init").about("Initialize the cache files"))
        .get_matches();

    match matches.subcommand() {
        Some(("client", sub_matches)) => client(sub_matches),
        Some(("server", sub_matches)) => server(sub_matches),
        Some(("init", sub_matches)) => init(sub_matches),
        _ => unreachable!(),
    }
}

fn client(matches: &ArgMatches) {
    let mut directory;
    let mut question_no;
    let mut solc_version;
    let mut job_id;
    let mut timeout;
    let mut connection_str;

    directory = String::from("usercode"); // default value
    let dir_env = env::var("DIRECTORY");
    if dir_env.is_ok() {
        directory = dir_env.unwrap();
    }
    if matches.value_source("directory") == Some(ValueSource::CommandLine) {
        directory = matches.get_one::<String>("directory").unwrap().to_string();
    }

    question_no = String::from("");
    let question_no_env = env::var("QUESTION_NO");
    if question_no_env.is_ok() {
        question_no = question_no_env.unwrap();
    }
    if matches.value_source("question-no") == Some(ValueSource::CommandLine) {
        question_no = matches
            .get_one::<String>("question-no")
            .unwrap()
            .to_string();
    }

    if question_no.len() == 0 {
        println!("Question no should not be empty, please set it via env QUESTION_NO or pass it by --question-no <question-no>");
        return;
    }

    solc_version = String::from("0.8.20");
    let solc_env = env::var("SOLC_VERSION");
    if solc_env.is_ok() {
        solc_version = solc_env.unwrap();
    }
    if matches.value_source("solc-version") == Some(ValueSource::CommandLine) {
        solc_version = matches
            .get_one::<String>("solc-version")
            .unwrap()
            .to_string();
    }

    job_id = String::from("");
    let job_id_env = env::var("JOB_ID");
    if job_id_env.is_ok() {
        job_id = job_id_env.unwrap();
    }
    if matches.value_source("job-id") == Some(ValueSource::CommandLine) {
        job_id = matches.get_one::<String>("job-id").unwrap().to_string();
    }

    if job_id.len() == 0 {
        println!("Job id no should not be empty, please set it via env JOB_ID or pass it by --job-id <job-id>");
        return;
    }

    timeout = String::from("5");
    let timeout_env = env::var("TIMEOUT");
    if timeout_env.is_ok() {
        timeout = timeout_env.unwrap();
    }
    if matches.value_source("timeout") == Some(ValueSource::CommandLine) {
        timeout = matches.get_one::<String>("timeout").unwrap().to_string();
    }

    connection_str = String::from("127.0.0.1:6379");
    let connection_str_env = env::var("CONNECTION_STR");
    if connection_str_env.is_ok() {
        connection_str = connection_str_env.unwrap();
    }
    if matches.value_source("connection-str") == Some(ValueSource::CommandLine) {
        connection_str = matches
            .get_one::<String>("connection-str")
            .unwrap()
            .to_string();
    }

    // print!("{}", dir);
    let res = client::request(
        directory,
        solc_version,
        question_no,
        job_id,
        timeout,
        connection_str,
    );
    if res.is_err() {
        print!("{:?}", res.unwrap_err());
    }
}

fn server(matches: &ArgMatches) {
    // config priority cmd param > env > default, so read from default
    let mut thread_num_holder: Option<String> = None;
    let mut redis_host_holder: Option<String> = None;
    let mut redis_prefix_holder: Option<String> = None;
    let mut redis_worker_dir_holder: Option<String> = None;
    let mut redis_list_name_holder: Option<String> = None;

    if matches.value_source("thread-num").unwrap() == ValueSource::CommandLine {
        thread_num_holder = Some(matches.get_one::<String>("thread-num").unwrap().to_string());
    }

    if matches.value_source("redis-host").unwrap() == ValueSource::CommandLine {
        redis_host_holder = Some(matches.get_one::<String>("redis-host").unwrap().to_string());
    }

    if matches.value_source("redis-prefix").unwrap() == ValueSource::CommandLine {
        redis_prefix_holder = Some(
            matches
                .get_one::<String>("redis-prefix")
                .unwrap()
                .to_string(),
        );
    }

    if matches.value_source("worker-dir").unwrap() == ValueSource::CommandLine {
        redis_worker_dir_holder =
            Some(matches.get_one::<String>("worker-dir").unwrap().to_string());
    }

    if matches.value_source("redis-list-name").unwrap() == ValueSource::CommandLine {
        redis_list_name_holder = Some(
            matches
                .get_one::<String>("redis-list-name")
                .unwrap()
                .to_string(),
        );
    }

    let _thread_num_res = env::var("THREAD_NUM");
    if _thread_num_res.is_ok() && thread_num_holder.is_none() {
        thread_num_holder = Some(_thread_num_res.unwrap());
    }

    let _redis_host_res = env::var("REDIS_HOST");
    if _redis_host_res.is_ok() && redis_host_holder.is_none() {
        redis_host_holder = Some(_redis_host_res.unwrap());
    }

    let _redis_prefix_res = env::var("REDIS_PREFIX");
    if _redis_prefix_res.is_ok() && redis_prefix_holder.is_none() {
        redis_prefix_holder = Some(_redis_prefix_res.unwrap());
    }

    let _redis_worker_dir_res = env::var("REDIS_WORKER_DIR");
    if _redis_worker_dir_res.is_ok() && redis_worker_dir_holder.is_none() {
        redis_worker_dir_holder = Some(_redis_worker_dir_res.unwrap());
    }

    let _redis_list_name_res = env::var("REDIS_LIST_NAME");
    if _redis_list_name_res.is_ok() && redis_list_name_holder.is_none() {
        redis_list_name_holder = Some(_redis_list_name_res.unwrap());
    }

    if thread_num_holder.is_none() {
        thread_num_holder = Some(matches.get_one::<String>("thread-num").unwrap().to_string());
    }

    if redis_host_holder.is_none() {
        redis_host_holder = Some(matches.get_one::<String>("redis-host").unwrap().to_string());
    }

    if redis_prefix_holder.is_none() {
        redis_prefix_holder = Some(
            matches
                .get_one::<String>("redis-prefix")
                .unwrap()
                .to_string(),
        );
    }

    if redis_worker_dir_holder.is_none() {
        redis_worker_dir_holder =
            Some(matches.get_one::<String>("worker-dir").unwrap().to_string());
    }

    if redis_list_name_holder.is_none() {
        redis_list_name_holder = Some(
            matches
                .get_one::<String>("redis-list-name")
                .unwrap()
                .to_string(),
        );
    }

    let thread_num = thread_num_holder.unwrap();
    let redis_host = redis_host_holder.unwrap();
    let redis_prefix = redis_prefix_holder.unwrap();
    let redis_worker_dir = redis_worker_dir_holder.unwrap();
    let redis_list_name = redis_list_name_holder.unwrap();

    print!(
        "{} {} {} {} {}",
        thread_num, redis_host, redis_prefix, redis_worker_dir, redis_list_name
    );

    server::start(
        thread_num.parse::<i32>().unwrap(),
        redis_host.as_str(),
        redis_prefix.as_str(),
        redis_worker_dir.as_str(),
        redis_list_name.as_str(),
    );
}

fn init(matches: &ArgMatches) {
    let _ = server::init_cache_file();
}
