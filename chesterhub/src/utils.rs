use rocket::http::Status;
use std::fs;
use uuid::Uuid;

const REPOSITORY: &str = "./repositorys";

pub fn create_folder(folder_name: &str) -> Status {
    let actually_folder = folder_name.split("/").last().unwrap();

    let folder_path = format!("{REPOSITORY}/{actually_folder}");
    let _ = fs::create_dir(folder_path.clone());
    let _ = fs::File::create(format!("{folder_path}/logs.txt"));
    Status::Ok
}

pub fn create_commit(folder_name: &str) -> String {
    let uid = Uuid::new_v4();

    let _ = fs::create_dir(format!("{folder_name}/{uid}"));
    format!("{uid}")
}
