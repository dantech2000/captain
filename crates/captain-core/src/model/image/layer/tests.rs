use super::{ImageLayer, largest_layer_size};

fn layer(created_by: &str, size: u64) -> ImageLayer {
    ImageLayer {
        id: "<missing>".into(),
        created_by: created_by.into(),
        size,
        ..ImageLayer::default()
    }
}

#[test]
fn classic_metadata_steps_drop_the_shell() {
    assert_eq!(
        layer("/bin/sh -c #(nop)  CMD [\"sh\"]", 0).command(),
        "CMD [\"sh\"]"
    );
    assert_eq!(
        layer("/bin/sh -c #(nop) ADD file:abc in / ", 0).command(),
        "ADD file:abc in /"
    );
}

#[test]
fn classic_shell_steps_become_run() {
    assert_eq!(
        layer("/bin/sh -c apk add   --no-cache curl", 0).command(),
        "RUN apk add --no-cache curl"
    );
    assert_eq!(
        layer("|1 VERSION=1.2 /bin/sh -c make install", 0).command(),
        "RUN make install"
    );
}

#[test]
fn buildkit_steps_drop_the_marker() {
    assert_eq!(
        layer("RUN /bin/sh -c apt-get update # buildkit", 0).command(),
        "RUN apt-get update"
    );
    assert_eq!(
        layer("ENTRYPOINT [\"/entry.sh\"]", 0).command(),
        "ENTRYPOINT [\"/entry.sh\"]"
    );
    assert_eq!(layer("COPY . /app # buildkit", 0).command(), "COPY . /app");
    assert_eq!(layer("", 0).command(), "");
}

#[test]
fn sizes_scale_to_the_largest_layer() {
    let layers = [layer("a", 10), layer("b", 40), layer("c", 0)];
    let largest = largest_layer_size(&layers);
    assert_eq!(largest, 40);
    assert_eq!(layers[0].size_fraction(largest), 0.25);
    assert_eq!(layers[1].size_fraction(largest), 1.0);
    assert_eq!(layers[2].size_fraction(largest), 0.0);
    assert_eq!(layers[0].size_fraction(0), 0.0);
    assert_eq!(largest_layer_size(&[]), 0);
}
