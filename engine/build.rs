use std::{fs, path::Path, str::FromStr};

use reqwest::Url;

fn main() {
    let net_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("net/");

    println!("cargo::rerun-if-env-changed=EVALFILE");
    println!("cargo::rerun-if-env-changed=CARGO_MANIFEST_DIR");
    println!("cargo::rerun-if-changed={}", net_dir.to_str().unwrap());

    if !net_dir.exists() {
        fs::create_dir(&net_dir).unwrap()
    }

    let net_path = net_dir.join(std::env::var("EVALFILE").unwrap());
    if !net_path.exists() {
        reqwest::blocking::get(
            Url::from_str("https://data.stockfishchess.org/nn/")
                .unwrap()
                .join(&std::env::var("EVALFILE").unwrap())
                .unwrap(),
        )
        .expect("request failed")
        .copy_to(&mut fs::File::create(&net_path).expect("failed to create file"))
        .unwrap();
    }
    println!("cargo::rustc-env=EVALFILE={}", net_path.to_str().unwrap());
}
