use serde::Deserialize;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use url::Url;
use walkdir::WalkDir;


#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ModInfo {
    #[serde(rename = "_sName")]
    pub name: String,

    #[serde(rename = "_idRow")]
    pub id: i32,

    #[serde(rename = "_aSubmitter")]
    pub submitter: Submitter,
    
    #[serde(rename = "_sDescription")]
    pub description: Option<String>,

    #[serde(rename = "_sText")]
    pub text: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct Submitter {
    #[serde(rename = "_sName")]
    pub name: String,
    
    #[serde(rename = "_sAvatarUrl")]
    pub avatar_url: String,

}

#[derive(Debug, Deserialize)]
struct FileResponse {
    #[serde(rename = "_aFiles")]
    files: Vec<FileInfo>,
}

#[derive(Debug, Deserialize)]
struct FileInfo {
    #[serde(rename = "_sFile")]
    filename: String,

    #[serde(rename = "_sDownloadUrl")]
    download_url: String,
}

#[derive(Debug, Deserialize)]
struct SubmissionsResponse {
    #[serde(rename = "_aRecords")]
    records: Vec<ModInfo>,
}

fn get_mod_info_from_url(url: &str) -> Result<ModInfo, &'static str> {
    let url_parsed = Url::parse(url).unwrap();
    let id = url_parsed.path_segments().unwrap().last().unwrap();

    let info: ModInfo = reqwest::blocking::get(format!(
        "https://gamebanana.com/apiv11/Mod/{}/ProfilePage",
        id
    ))
    .unwrap()
    .json()
    .unwrap();

    Ok(info)
}

pub fn download_mod(mod_info: &ModInfo) {
    let client = reqwest::blocking::Client::new();
    let file_response: FileResponse = client
        .get(format!(
            "https://gamebanana.com/apiv11/Mod/{}/ProfilePage",
            mod_info.id
        ))
        .send()
        .unwrap()
        .json()
        .unwrap();

    let file_info = &file_response.files.get(0).unwrap();

    let downloaded_file = client
        .get(&file_info.download_url)
        .send()
        .unwrap()
        .bytes()
        .unwrap();

    let mod_download_path_string = format!("./data/download/{}", &mod_info.id);
    let mod_download_path = Path::new(&mod_download_path_string);

    let _ = fs::create_dir_all(mod_download_path);

    let mod_pathbuf = PathBuf::from(mod_download_path);

    let mod_archive_pathbuf = mod_pathbuf.join(&file_info.filename);
    let mod_archive_path = mod_archive_pathbuf.as_path();

    fs::write(mod_archive_path, &downloaded_file).unwrap();

    let mod_extracted_dir = Path::new(&file_info.filename)
        .file_stem()
        .unwrap()
        .to_string_lossy();

    let _ = fs::create_dir(format!(
        "{}/{}",
        mod_download_path.to_string_lossy(),
        mod_extracted_dir
    ));

    let mod_extracted_pathbuf = mod_pathbuf.join(mod_extracted_dir.to_string());
    let mod_extracted_path = mod_extracted_pathbuf.as_path().canonicalize().unwrap();

    let mut command = Command::new("7z");
    command
        .arg("x")
        .arg(format!(
            "-o{}",
            mod_extracted_path.to_string_lossy().replace(r"\\?\", "")
        ))
        .arg(format!(
            "{}",
            mod_archive_path
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .replace(r"\\?\", "")
        ));

    let _ = command.output().unwrap();
}

pub fn install_mod(mod_info: &ModInfo, data_win_path: PathBuf) {
    // search mod folder for xdelta file
    let find_file_path = find_file(mod_info, "xdelta").unwrap();
    let xdelta_path = find_file_path.as_path().canonicalize().unwrap();

    println!("{:?}", xdelta_path);

    let data_win_backup_path = data_win_path.parent().unwrap().join("data.win.bak");

    if !fs::exists(&data_win_backup_path).unwrap() {
        std::fs::copy(&data_win_path, data_win_backup_path.as_path()).unwrap();
        println!("Created backup copy at data.win.bak.")
    }

    let mut command = Command::new("xdelta");
    command
        .arg("-d")
        .arg("-f")
        .arg("-s")
        .arg(data_win_backup_path)
        .arg(xdelta_path)
        .arg(data_win_path);

    let output = command.output().unwrap();

    println!("{:#?}", output);
}

fn find_file(mod_info: &ModInfo, ext: &str) -> Option<PathBuf> {
    let mod_download_path_string = format!("./data/download/{}", &mod_info.id);
    let mod_download_path = Path::new(&mod_download_path_string);

    for entry in WalkDir::new(mod_download_path)
        .into_iter()
        .filter_map(|x| x.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension().and_then(|x| x.to_str()) {
                if extension == ext {
                    return Some(path.to_path_buf());
                }
            }
        }
    }

    None
}

pub fn get_all_mods() -> Vec<ModInfo> {
    let client = reqwest::blocking::Client::new();
    let submissions_response: SubmissionsResponse = client
        .get("https://gamebanana.com/apiv11/Game/21841/Subfeed?_nPage=1&_sSort=new")
        .send()
        .unwrap()
        .json()
        .unwrap();
    return submissions_response.records.iter().map(|x| get_mod_info_from_url(format!("https://gamebanana.com/mods/{}", x.id).as_str()).unwrap()).collect()
}
