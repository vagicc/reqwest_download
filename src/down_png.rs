use error_chain::error_chain;
use std::fs::File;
use std::io::copy;
use std::io::Read;
use std::path::PathBuf;
use tempfile::Builder;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("下载文件到临时目录!");
    // let tmp_dir = Builder::new().prefix("example").tempdir()?;
    let tmp_dir = Builder::new().tempdir()?;
    let target = "https://www.rust-lang.org/logos/rust-logo-512x512.png";
    let mut response = reqwest::get(target).await?;
    println!("请求返回状态码：{}", response.status());

    if !response.status().is_success() {
        println!("get请求失败：{:#?}", response);
        return Ok(());
    }

    // let mut dest = {
    //     let fname = response
    //         .url()
    //         .path_segments()
    //         .and_then(|segments| segments.last())
    //         .and_then(|name| if name.is_empty() { None } else { Some(name) })
    //         .unwrap_or("tmp.bin");

    //     println!("下载文件名：‘{}’", fname);
    //     let f = format!("/home/luck/{}", fname);
    //     File::create(f)?

    //     // let fname = tmp_dir.path().join(fname);
    //     // let path = PathBuf::from(f);
    //     // println!("will be located under: {:?}", fname);
    //     // File::create(fname)?
    // };

    // let content = response.text().await?;
    // let tk = copy(&mut content.as_bytes(), &mut dest);   //这个方法生成的图片打不开

    let file = "luck.png";
    println!("写入文件：{}", file);
    let result = save(file, &mut response).await;

    println!("POST 文件到 paste-rs");
    let paste_api = "https://paste.rs";
    let mut file = File::open("Cargo.toml")?;

    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let client = reqwest::Client::new();
    let response = client.post(paste_api).body(contents).send().await?;
    let response_text = response.text().await?;
    println!("Your paste is located at(你上传的文件的URL): {}", response_text);

    Ok(())
}

async fn save(filename: &str, response: &mut reqwest::Response) -> Result<()> {
    let mut options = OpenOptions::new();
    let mut file = options
        .append(true)
        .create(true)
        .read(true)
        .open(filename)
        .await?;

    while let Some(chunk) = &response.chunk().await.expect("Failed") {
        match file.write_all(&chunk).await {
            Ok(_) => {
                println!("写入成功");
            }
            Err(e) => {
                println!("写入失败：{:#?}", e);
                return Ok(());
            }
        }
    }
    Ok(())
}
