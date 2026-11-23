#[allow(dead_code)]
mod installer {
    include!("../src/global_skill.rs");

    #[test]
    fn failed_write_keeps_partial_file_and_refuses_retry() {
        let root =
            std::env::temp_dir().join(format!("snipe-skill-cleanup-{}", uuid::Uuid::new_v4()));
        let result = install(&root, |file| {
            file.write_all(b"partial")?;
            Err(std::io::Error::other("injected failure"))
        });
        assert!(result.is_err());
        let file = root.join("snipe-code/SKILL.md");
        assert_eq!(std::fs::read(&file).unwrap(), b"partial");
        assert!(install(&root, |file| file.write_all(b"replacement")).is_err());
        assert_eq!(std::fs::read(&file).unwrap(), b"partial");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn existing_directory_is_never_removed() {
        let root =
            std::env::temp_dir().join(format!("snipe-skill-existing-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(root.join("snipe-code")).unwrap();
        std::fs::write(root.join("snipe-code/keep"), b"existing").unwrap();
        assert!(install(&root, |file| file.write_all(b"replacement")).is_err());
        assert_eq!(
            std::fs::read(root.join("snipe-code/keep")).unwrap(),
            b"existing"
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
