#[macro_use]
extern crate rocket;
use chrono::Utc;
use rocket::form::Form;
use rocket::http::Status;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
mod structs;
mod utils;

#[post("/create/repository", data = "<name>")]
fn create_repo(name: String) -> Status {
    if name != "" {
        let status = utils::create_folder(&name);

        return status;
    }

    Status::NotAcceptable
}

#[post("/create/repository/<name>/commit")]
fn create_commit(name: String) -> String {
    let folder_path = format!("./repositorys/{name}");
    utils::create_commit(&folder_path)
}

#[post("/repository/upload/file", data = "<upload>")]
async fn receive_file(mut upload: Form<structs::Commit<'_>>) -> Status {
    let log_path = format!("./repositorys/{}/logs.txt", upload.repo_name());
    let uploaded_file_path = format!(
        "./repositorys/{}/{}/temp.zip",
        upload.repo_name(),
        upload.commit_uid()
    );
    upload.save_file(&uploaded_file_path).await;

    let mut log = OpenOptions::new().append(true).open(log_path).unwrap();
    let date = Utc::now();

    let log_data = format!("{}+{}+{}\n", upload.commit_uid(), date, upload.msg_string());

    let _result = log.write_all(log_data.as_bytes());

    Status::Ok
}

// in a serious way -> should return a json
#[get("/repository/<name>/log")]
fn return_log(name: String) -> Option<File> {
    let log_path = format!("./repositorys/{name}/logs.txt");

    File::open(&log_path).ok()
}

#[get("/repository/<name>/commit/<uid>")]
fn return_commit(name: String, uid: String) -> Option<File> {
    let commit_file_path = format!("./repositorys/{name}/{uid}/temp.zip");

    File::open(&commit_file_path).ok()
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount(
        "/",
        routes!(
            create_repo,
            receive_file,
            create_commit,
            return_log,
            return_commit
        ),
    )
}
