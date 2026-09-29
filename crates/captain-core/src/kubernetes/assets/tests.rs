use super::{download_url, expected_sha256};

#[test]
fn finds_the_hash_of_one_file() {
    let listing = "d842  k3s-airgap-images-arm64.tar\n\
        9D3C4C2197BCF857CA17633AA393BAD683CC982DDD408620F93036A3CCA953B5  k3s-airgap-images-arm64.tar.zst\n\
        c920706346d5ad4e5cd3c7bf1bb09ce71ebe07fec829e513e40f1caf98aed8bb *k3s-arm64\n";
    assert_eq!(
        expected_sha256(listing, "k3s-arm64").unwrap(),
        "c920706346d5ad4e5cd3c7bf1bb09ce71ebe07fec829e513e40f1caf98aed8bb"
    );
    assert!(
        expected_sha256(listing, "k3s-airgap-images-arm64.tar.zst")
            .unwrap()
            .starts_with("9d3c")
    );
    assert_eq!(
        expected_sha256(listing, "k3s-airgap-images-arm64.tar"),
        None
    );
}

#[test]
fn escapes_the_plus_in_the_url() {
    assert_eq!(
        download_url("v1.36.4+k3s1", "k3s-arm64"),
        "https://github.com/k3s-io/k3s/releases/download/v1.36.4%2Bk3s1/k3s-arm64"
    );
}
