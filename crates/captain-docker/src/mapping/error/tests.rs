use bollard::errors::Error;

use super::found;

fn server_error(status_code: u16) -> Result<(), Error> {
    Err(Error::DockerResponseServerError {
        status_code,
        message: "engine error".into(),
    })
}

#[test]
fn only_a_404_proves_the_object_does_not_exist() {
    assert_eq!(found(server_error(404)).unwrap(), None);
    assert!(found(server_error(500)).is_err());
}
