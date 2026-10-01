use super::*;

#[test]
fn words_split_like_a_shell() {
    let text = "docker run -e 'A=one two' -e \"B=say \\\"hi\\\" \\$HOME\" \\\n  -e C=x\\ y \"\" \\\r\n nginx";

    assert_eq!(
        split_words(text).unwrap(),
        [
            "docker",
            "run",
            "-e",
            "A=one two",
            "-e",
            "B=say \"hi\" $HOME",
            "-e",
            "C=x y",
            "",
            "nginx"
        ]
    );
    assert!(split_words("docker run -e 'A=b nginx").is_err());
    assert!(split_words("docker run -e \"A=b nginx").is_err());
}
