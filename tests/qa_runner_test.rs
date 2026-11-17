use snipe_code::qa_runner::{ExecutionMode, ManifestScope, Mount, RunManifest};

fn scope() -> ManifestScope {
    ManifestScope {
        command: vec!["npm".into(), "test".into()],
        cwd: "/project".into(),
        target: "database target unknown; endpoint redacted".into(),
        env_names: vec!["PATH".into(), "CI".into()],
        mounts: vec![Mount {
            source: "/project".into(),
            destination: "/work".into(),
            access: "read-only".into(),
        }],
        filesystem: "unrestricted host".into(),
        process: "unrestricted host".into(),
        network: "unrestricted host".into(),
        output: "/project/results".into(),
        timeout_seconds: 60,
        install_effects: vec![],
        execution_mode: ExecutionMode::Host,
    }
}

#[test]
fn manifest_is_a_deterministic_owned_snapshot() {
    let mut input = scope();
    let manifest = RunManifest::new(input.clone()).unwrap();
    assert_eq!(
        manifest.hash(),
        RunManifest::new(input.clone()).unwrap().hash()
    );
    input.command.push("new-argument".into());
    assert_ne!(manifest.hash(), RunManifest::new(input).unwrap().hash());
    assert_eq!(manifest.scope().command, ["npm", "test"]);
    assert!(
        manifest
            .preview()
            .contains("Host execution is not sandboxed")
    );
    assert!(manifest.preview().contains("database target unknown"));
}

#[test]
fn environment_values_and_incomplete_scope_are_rejected() {
    let mut input = scope();
    input.env_names.push("TOKEN=secret".into());
    assert!(RunManifest::new(input).is_err());
    let mut input = scope();
    input.network.clear();
    assert!(RunManifest::new(input).is_err());
    let mut input = scope();
    input.timeout_seconds = 0;
    assert!(RunManifest::new(input).is_err());
}

#[test]
fn supplied_db_settings_fail_closed_without_claiming_local_isolation() {
    use snipe_code::qa_runner::{DbPolicy, DbSettings, classify_db};
    assert_eq!(classify_db(None), DbPolicy::Unknown);
    assert!(DbPolicy::Unknown.disclosure().contains("unknown"));
    assert!(DbPolicy::Unknown.disclosure().contains("approval"));
    for (host, name, environment, expected) in [
        ("localhost", "app_test", "test", DbPolicy::LocalUnverified),
        ("127.0.0.1", "app_test", "test", DbPolicy::LocalUnverified),
        ("::1", "app_test", "test", DbPolicy::LocalUnverified),
        ("db.example.com", "app_test", "test", DbPolicy::Remote),
        (
            "localhost.attacker.example",
            "app_test",
            "test",
            DbPolicy::Remote,
        ),
        ("192.168.1.10", "app_test", "test", DbPolicy::Remote),
        (
            "localhost",
            "APP_PRODUCTION",
            "test",
            DbPolicy::ProductionLike,
        ),
        (
            "localhost",
            "app_test",
            "Production",
            DbPolicy::ProductionLike,
        ),
        ("localhost", "main", "test", DbPolicy::ProductionLike),
        ("", "app_test", "test", DbPolicy::Ambiguous),
        ("localhost", "", "test", DbPolicy::Ambiguous),
        (
            "postgres://user:secret@localhost",
            "app_test",
            "test",
            DbPolicy::Ambiguous,
        ),
        ("localhost,remote", "app_test", "test", DbPolicy::Ambiguous),
    ] {
        assert_eq!(
            classify_db(Some(DbSettings {
                host,
                name,
                environment
            })),
            expected,
            "{expected:?}"
        );
    }
    assert!(
        DbPolicy::LocalUnverified
            .disclosure()
            .contains("not proof of isolation")
    );
}

#[test]
fn public_fake_seam_without_approval_never_calls_callback() {
    use snipe_code::qa_runner::{DbPolicy, RunStatus, run_fake};
    let mut input = scope();
    input.target = DbPolicy::Unknown.disclosure().into();
    let manifest = RunManifest::new(input).unwrap();
    let result = run_fake(&manifest, DbPolicy::Unknown, None, |_| {
        panic!("unapproved fake callback must not run")
    });
    assert_eq!(result.status, RunStatus::Blocked);
    assert_eq!(result.exit_code, None);
    assert!(!result.approval_matched);
    assert!(result.artifacts.is_empty());
    assert_eq!(result.stdout, "");
    assert_eq!(result.stderr, "");
}
