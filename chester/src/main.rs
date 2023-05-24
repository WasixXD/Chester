use clap::Parser;
use colored::*;
use comfy_table::Table;
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Cursor;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default, Debug, Parser)]
struct Args {
    #[arg(short, long)]
    init: bool,

    #[arg(short, long, default_value_t = false)]
    commit: bool,

    #[arg(short, long, default_value_t=String::new())]
    add: String,

    #[arg(short, long, default_value_t=String::new())]
    msg: String,

    #[arg(short, long, default_value_t = false)]
    log: bool,

    #[arg(short, long, default_value_t=String::new())]
    pull: String,
}

fn path_exist(path: &str) -> bool {
    fs::metadata(path).is_ok()
}

fn get_dir_files(dir_path: &str) -> Vec<String> {
    let dir_files = fs::read_dir(dir_path).unwrap();
    let mut result = Vec::new();
    for file in dir_files {
        let path = file.unwrap().path().display().to_string();
        let meta = fs::metadata(path.clone()).unwrap();
        if meta.is_dir() {
            for i in get_dir_files(&path) {
                result.push(i)
            }
        }
        result.push(path);
    }
    result
}

fn current_dir(env: String) -> String {
    env.split("/").last().unwrap().to_string()
}

fn add_files_to_commit(files_paths: &str) {
    let stage_path = ".chester/stage.txt";
    let folder_path = ".chester/folders.txt";
    let meta = fs::metadata(files_paths).unwrap();

    let mut c: usize = 0;

    let mut stage = OpenOptions::new().append(true).open(stage_path).unwrap();
    let mut folder = OpenOptions::new().append(true).open(folder_path).unwrap();
    if meta.is_dir() {
        let files = get_dir_files(files_paths);
        for file in files {
            let dot_files = &file[1..3].contains(".");
            if !dot_files {
                let file_meta = fs::metadata(file.clone()).unwrap();
                if file_meta.is_dir() {
                    let content = format!("{file}\n");
                    let _ = folder.write_all(&content.as_bytes());

                    c += 1;
                } else {
                    let content = format!("{file}\n");
                    let _ = stage.write_all(&content.as_bytes());
                    c += 1;
                }
            }
        }
        print!("{c} ");
        print!("{}", "Files added!".green().bold());
    } else {
        let content = format!("{files_paths}\n");

        let _ = stage.write_all(&content.as_bytes());
        println!("{}", "File added!".green().bold());
    }
}

async fn commit(msg: String) {
    let stage_path = "./.chester/stage.txt";
    let folder_path = "./.chester/folders.txt";
    let commits_path = "./.chester/commit/";

    let content_files = fs::read_to_string(stage_path).unwrap();
    let content_folder = fs::read_to_string(folder_path).unwrap();

    let current_dir = current_dir(env::current_dir().unwrap().display().to_string());
    let files = content_files.split('\n');
    let folders = content_folder.split('\n');
    let zip_commit_path = "./.chester/commit.zip";

    for folder in folders {
        let _ = fs::create_dir(format!("{commits_path}{folder}"));
    }
    for file in files {
        let _ = fs::copy(file, format!("{commits_path}{file}"));
    }

    println!("{}", "Zipping folders...".cyan());
    let _dmp = Command::new("zip")
        .arg("-5")
        .args(["-r", zip_commit_path])
        .arg("./.chester/commit/")
        .arg("commit")
        .output();
    let client = reqwest::Client::new();

    let file = fs::read(zip_commit_path).unwrap();
    let url = format!("http://127.0.0.1:8000/create/repository/{current_dir}/commit");
    let commit_uid = client.post(url).send().await.unwrap().text().await.unwrap();

    let file_part = reqwest::multipart::Part::bytes(file)
        .file_name("folder.zip")
        .mime_str("application/zip")
        .unwrap();

    let form = reqwest::multipart::Form::new()
        .part("zip", file_part)
        .text("repository", current_dir)
        .text("commit", commit_uid)
        .text("mensage", msg);

    let url = format!("http://127.0.0.1:8000/repository/upload/file");
    let _resp = client.post(url).multipart(form).send().await;

    println!("{}", "All files commited!".green().bold());

    // need to delete because in other commits it gets too much error
    let _ = fs::remove_file(zip_commit_path);
    let _ = fs::remove_dir_all("./.chester/commit");
    let _ = fs::create_dir("./.chester/commit");
}

fn create_folder() {
    let chester_path = "./.chester";
    let stage_path = format!("{chester_path}/stage.txt");
    let folder_path = format!("{chester_path}/folders.txt");
    let commit_path = format!("{chester_path}/commit");

    if !path_exist(chester_path) {
        let _ = fs::create_dir(chester_path).expect("Could not create .chester dir");
        let _ = fs::File::create(stage_path);
        let _ = fs::File::create(folder_path);
        let _ = fs::create_dir(commit_path);

        println!("{}", "A new chester has appeared!".green().bold());
    } else {
        println!("{}", "Folder already created!".red().bold());
    }
}

async fn log() {
    // DRY???
    let current_dir = current_dir(env::current_dir().unwrap().display().to_string());
    let url = format!("http://127.0.0.1:8000/repository/{current_dir}/log");
    let result = reqwest::get(url).await.unwrap().text().await.unwrap();

    let mut table = Table::new();
    table.set_header(vec!["Commit UID", "Date(UTC)", "Mensage"]);

    let lines = result.lines();
    for line in lines {
        let content = line.split("+");
        let row: Vec<_> = content.collect();
        table.add_row(row);
    }

    println!("{table}");
}

async fn pull(uid: String) {
    let current_dir = current_dir(env::current_dir().unwrap().display().to_string());
    let url = format!("http://127.0.0.1:8000/repository/{current_dir}/commit/{uid}");
    let result = reqwest::get(url).await.unwrap().bytes().await.unwrap();

    let _unzip = zip_extract::extract(Cursor::new(result), &PathBuf::from("./"), true);

    println!(
        "{}\n{}",
        "Files pulled!".purple().bold(),
        "Check ./commit".italic().cyan()
    );
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    if args.init {
        let current_dir = env::current_dir().unwrap().display().to_string();
        let client = reqwest::Client::new();
        let _ = client
            .post("http://127.0.0.1:8000/create/repository")
            .body(current_dir)
            .send()
            .await
            .unwrap()
            .text()
            .await;

        create_folder();
    } else if args.commit && args.msg != "" {
        commit(args.msg).await;
    } else if args.add != "" {
        add_files_to_commit(&args.add)
    } else if args.log {
        log().await;
    } else if args.pull != "" {
        pull(args.pull).await;
    }
}
