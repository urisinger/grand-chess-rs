use std::{
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
};

use reqwest::Url;

fn main() {
    let net_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("net/");

    println!("cargo:rerun-if-changed={}", net_dir.to_str().unwrap());
    if !net_dir.exists() {
        fs::create_dir(&net_dir).unwrap()
    }

    let net_path = net_dir.join(std::env::var("EVALFILE").unwrap());
    if !net_path.exists() {
        let resp = reqwest::blocking::get(
            Url::from_str("https://tests.stockfishchess.org/api/nn/")
                .unwrap()
                .join(&std::env::var("EVALFILE").unwrap())
                .unwrap(),
        )
        .expect("request failed");
        let body = resp.bytes().expect("body invalid");
        let mut out = fs::File::create(&net_path).expect("failed to create file");
        io::copy(&mut io::Cursor::new(body), &mut out).expect("failed to copy content");
    }
    println!("cargo::rustc-env=EVALFILE={}", net_path.to_str().unwrap());
}
