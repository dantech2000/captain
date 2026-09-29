use super::*;

#[test]
fn helper_output_is_a_password_or_an_identity_token() {
    let server = "https://index.docker.io/v1/";
    let password = RegistryAuth::from_helper(
        r#"{"ServerURL":"https://index.docker.io/v1/","Username":"dan","Secret":"pw"}"#,
        server,
    )
    .unwrap();
    assert_eq!(password.username.as_deref(), Some("dan"));
    assert_eq!(password.password.as_deref(), Some("pw"));

    let token =
        RegistryAuth::from_helper(r#"{"Username":"<token>","Secret":"tok"}"#, server).unwrap();
    assert_eq!(token.identity_token.as_deref(), Some("tok"));
    assert_eq!(token.username, None);

    assert_eq!(
        RegistryAuth::from_helper("credentials not found in native keychain", server),
        None
    );
}
