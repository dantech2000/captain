use super::*;

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("captain-{name}-{}", std::process::id()));
    fs::remove_dir_all(&dir).ok();
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn shop(dir: &str) -> KnownProject {
    KnownProject {
        name: "shop".into(),
        dir: PathBuf::from(dir),
        files: vec!["compose.yaml".into()],
        added: 1_790_000_000,
    }
}

#[test]
fn a_saved_list_reads_back_a_missing_file_is_empty_and_a_broken_one_an_error() {
    let home = temp_dir("known-round-trip");
    let path = known_projects_path(&home);
    assert_eq!(
        KnownProjects::load(&path).unwrap(),
        KnownProjects::default()
    );

    let mut list = KnownProjects::default();
    list.add(shop("/code/shop")).unwrap();
    list.save(&path).unwrap();

    assert_eq!(KnownProjects::load(&path).unwrap(), list);
    fs::remove_dir_all(&home).ok();
    let home = temp_dir("known-broken");
    let path = known_projects_path(&home);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "{ not json").unwrap();

    assert!(matches!(
        KnownProjects::load(&path),
        Err(KnownProjectsError::Parse { .. })
    ));
    fs::remove_dir_all(&home).ok();
}

#[test]
fn add_updates_the_same_folder_refuses_another_and_remove_forgets() {
    let mut list = KnownProjects::default();
    list.add(shop("/code/shop")).unwrap();
    let mut again = shop("/code/shop");
    again.files.push("compose.override.yml".into());
    list.add(again.clone()).unwrap();
    assert_eq!(list.projects, vec![again]);

    assert!(matches!(
        list.add(shop("/other/shop")),
        Err(KnownProjectsError::NameTaken { .. })
    ));

    assert!(list.remove("shop"));
    assert!(list.projects.is_empty());
}
