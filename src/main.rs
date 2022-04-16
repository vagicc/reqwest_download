use error_chain::error_chain;
use reqwest::header::{HeaderValue, CONTENT_LENGTH, RANGE};
use reqwest::StatusCode;
use std::fs::File;
use std::str::FromStr;

error_chain! {
    foreign_links{
        Io(std::io::Error);
        Reqwest(reqwest::Error);
        Header(reqwest::header::ToStrError);
    }
}

struct PartialRangeIter {
    start: u64,
    end: u64,
    buffer_size: u32,
}

impl PartialRangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u32) -> Result<Self> {
        if buffer_size == 0 {
            Err("invalid buffer_size, give a value greater than zero.")?;
        }
        Ok(PartialRangeIter {
            start,
            end,
            buffer_size,
        })
    }
}

impl Iterator for PartialRangeIter {
    type Item = HeaderValue;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size as u64, self.end - self.start + 1);
            Some(
                HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1))
                    .expect("格式字符串出错!"),
            )
        }
    }
}

fn main() -> Result<()> {
    println!("使用 HTTP range 请求头进行部分下载!");
    let url = "https://httpbin.org/range/102400?duration=2";
    const CHUNK_SIZE: u32 = 102240;

    let client = reqwest::blocking::Client::new();
    let response = client.head(url).send()?;
    println!("请求返回状态码：{}", response.status());

    let length = response
        .headers()
        .get(CONTENT_LENGTH)
        .ok_or("响应不包括内容长度")?;
    let length = u64::from_str(length.to_str()?).map_err(|_| "内容长度无效的头")?;

    let mut output_file = File::create("download.bin")?;

    println!("开始下载 starting download ...");
    for range in PartialRangeIter::new(0, length - 1, CHUNK_SIZE)? {
        println!("range {:?}", range);
        let mut response = client.get(url).header(RANGE, range).send()?;

        let status = response.status();
        if !(status == StatusCode::OK) || status == StatusCode::PARTIAL_CONTENT {
            // error_chain::bail!("意想不到的服务器响应:{}", status);
        }

        std::io::copy(&mut response,&mut output_file)?;

    }

    let content=response.text()?;
    std::io::copy(&mut content.as_bytes(),&mut output_file)?;
    println!("Finished with success!");

    Ok(())
}
