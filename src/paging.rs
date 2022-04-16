use reqwest::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ApiResponse {
    dependencies: Vec<Dependency>,
    meta: Meta,
}

#[derive(Debug, Deserialize)]
struct Dependency {
    crate_id: String,
}

#[derive(Debug, Deserialize)]
struct Meta {
    total: u32,
}

struct ReverseDependencies {
    crate_id: String,
    dependencies: <Vec<Dependency> as IntoIterator>::IntoIter,
    client: reqwest::blocking::Client,
    page: u32,
    per_page: u32,
    total: u32,
}
impl ReverseDependencies {
    fn of(crate_id: &str) -> Result<Self> {
        Ok(ReverseDependencies {
            crate_id: crate_id.to_owned(),
            dependencies: vec![].into_iter(),
            client: reqwest::blocking::Client::new(),
            page: 0,
            total: 0,
            per_page: 100,
        })
    }

    fn try_next(&mut self) -> Result<Option<Dependency>> {
        if let Some(dep) = self.dependencies.next() {
            return Ok(Some(dep));
        }
        if self.page > 0 && self.page * self.per_page >= self.total {
            return Ok(None);
        }

        self.page += 1;
        let url = format!(
            "https://crates.io/api/v1/crates/{}/reverse_dependencies?page={}&per_page={}",
            self.crate_id, self.page, self.per_page
        );
        println!("请求： {}", url);

        let response = self
            .client
            .get(&url)
            .header("user-agent", "Rust-test")
            .send()
            .unwrap();

        println!("请求状态：{}", response.status());

        if !response.status().is_success(){
            return Ok(None);
        }

        let data=response.json::<ApiResponse>().unwrap();
        self.dependencies=data.dependencies.into_iter();
        self.total = data.meta.total;

        Ok(self.dependencies.next())
    }
}

impl Iterator for ReverseDependencies {
    type Item = Result<Dependency>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.try_next() {
            Ok(Some(dep)) => Some(Ok(dep)),
            Ok(None) => None,
            Err(err) => Some(Err(err)),
        }
    }
}

fn main() {
    println!("使用 RESTful API 分页!");
    for dep in ReverseDependencies::of("serde").unwrap() {
        println!("reverse dependency: {}", dep.unwrap().crate_id);
    }
}
