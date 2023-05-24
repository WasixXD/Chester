use rocket::fs::TempFile;
use rocket::serde::Serialize;

#[derive(FromForm, Debug)]
pub struct Commit<'a> {
    repository: String,
    mensage: String,
    commit: String,
    zip: TempFile<'a>,
}

impl Commit<'_> {
    pub fn repo_name(&self) -> String {
        format!("{}", self.repository)
    }

    pub fn msg_string(&self) -> String {
        format!("{}", self.mensage)
    }

    pub fn commit_uid(&self) -> String {
        format!("{}", self.commit)
    }
    pub async fn save_file(&mut self, path: &str) {
        let _result = self.zip.copy_to(path).await;
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(crate = "rocket::serde")]
pub struct Info {
    pub commit_uid: String,
    pub date: String,
    pub msg: String,
}

#[derive(Serialize, Debug)]
#[serde(crate = "rocket::serde")]
pub struct Response {
    pub data: Vec<Info>,
}
