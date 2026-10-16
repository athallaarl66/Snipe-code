//! The distribution contract: one source of truth for the release targets, the
//! documentation that must match it, the publication metadata, and installers that
//! fail closed.

use std::fs;
use std::path::PathBuf;

fn repo_file(relative: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

/// The release target list, as the workflow reads it.
fn release_targets() -> serde_json::Value {
    serde_json::from_str(&repo_file(".github/release-targets.json")).expect("release targets parse")
}

fn target_names() -> Vec<String> {
    release_targets()["targets"]
        .as_array()
        .expect("targets array")
        .iter()
        .map(|target| target["target"].as_str().expect("target name").to_string())
        .collect()
}

// --- 3.1 one source of truth for the matrix --------------------------------

#[test]
fn test_release_targets_expand_to_the_expected_matrix() {
    let targets = release_targets();
    let entries = targets["targets"].as_array().expect("targets array");

    assert_eq!(
        target_names(),
        [
            "x86_64-unknown-linux-gnu",
            "x86_64-pc-windows-msvc",
            "x86_64-apple-darwin",
            "aarch64-apple-darwin",
        ],
        "the built platform list must cover Linux, Windows, and both macOS architectures"
    );
    for entry in entries {
        for field in ["target", "runner", "archive", "binary", "platform"] {
            assert!(
                entry[field].is_string(),
                "target entry is missing {field}: {entry}"
            );
        }
        assert!(
            matches!(entry["archive"].as_str(), Some("tar.gz") | Some("zip")),
            "unsupported archive format: {entry}"
        );
        assert!(
            entry["runner"].as_str().expect("runner").contains("latest")
                || entry["runner"].as_str().expect("runner") == "macos-13",
            "unexpected runner: {entry}"
        );
    }

    let workflow = repo_file(".github/workflows/release.yml");
    assert!(
        workflow.contains("include: ${{ fromJSON(needs.targets.outputs.matrix) }}"),
        "the package matrix must be read from the target list"
    );
    assert!(
        workflow.contains(".github/release-targets.json"),
        "the workflow must read the target list"
    );
}

// --- 3.2 release validation is derived --------------------------------------

#[test]
fn test_release_validation_is_derived_from_the_target_list() {
    let workflow = repo_file(".github/workflows/release.yml");

    assert!(
        !workflow.contains("== 3"),
        "the archive-count assertion must not be a hardcoded literal"
    );
    assert!(
        workflow.contains("needs.targets.outputs.count"),
        "the expected count must come from the target list"
    );
    assert!(
        workflow.contains("jq -r '.targets[].target'"),
        "each expected target must be checked against the published archives"
    );
}

// --- 3.3 documentation matches what is built --------------------------------

#[test]
fn test_documented_platforms_match_the_built_targets() {
    let technical = repo_file("docs/features/technical.md");
    let readme = repo_file("README.md");

    for entry in release_targets()["targets"].as_array().expect("targets") {
        let target = entry["target"].as_str().expect("target");
        let platform = entry["platform"].as_str().expect("platform");
        assert!(
            technical.contains(target),
            "technical.md must document the built target {target}"
        );
        assert!(
            readme.contains(target),
            "README must document the built target {target}"
        );
        assert!(
            readme.contains(platform),
            "README must name {platform} for {target}"
        );
    }

    assert!(
        !technical.contains("snipe-code-macos-arm64\n"),
        "the stale archive name must be replaced by the real one"
    );
    assert_eq!(
        target_names().len(),
        4,
        "every built target must be documented, and nothing else claimed"
    );
}

// --- 5.1 publication metadata ----------------------------------------------

#[test]
fn test_cargo_manifest_carries_complete_publication_metadata() {
    let manifest = repo_file("Cargo.toml");
    for expected in [
        "description =",
        "license = \"MIT\"",
        "repository = \"https://github.com/athallaarl66/Snipe-code\"",
        "readme = \"README.md\"",
        "keywords = [",
        "categories = [",
    ] {
        assert!(
            manifest.contains(expected),
            "Cargo.toml is missing {expected}:\n{manifest}"
        );
    }

    let license = repo_file("LICENSE");
    assert!(
        license.starts_with("MIT License"),
        "the declared license must match the license file:\n{license}"
    );
}

// --- 5.2-5.4 crate publishing in the release workflow ----------------------

#[test]
fn test_release_publishes_the_crate_after_the_quality_gate() {
    let workflow = repo_file(".github/workflows/release.yml");

    let crate_job = workflow
        .split("  crate:")
        .nth(1)
        .expect("the release workflow must have a crate job");

    assert!(
        crate_job.contains("needs: quality"),
        "the crate job must depend on the quality gate:\n{crate_job}"
    );
    assert!(
        crate_job.contains("cargo publish"),
        "the crate job must publish the crate:\n{crate_job}"
    );
    assert!(
        crate_job.contains("api/v1/crates/snipe-code"),
        "the crate job must check the registry first, so a re-run converges:\n{crate_job}"
    );
    assert!(
        crate_job.contains("CARGO_REGISTRY_TOKEN is not configured"),
        "a missing token must fail loudly instead of skipping the publish"
    );
    assert!(
        workflow.contains("Tag must match the crate version"),
        "the release must assert the tag equals the crate version"
    );
    assert!(
        workflow.contains("cargo publish --dry-run"),
        "a packaging check must precede the publish"
    );
}

// --- 6.2-6.4 the README documents the install and analysis paths -----------

#[test]
fn test_readme_documents_analyze_and_its_limits() {
    let readme = repo_file("README.md");
    for expected in ["snipe-code analyze", "--json", "Limits", "scanned"] {
        assert!(readme.contains(expected), "README must document {expected}");
    }
    // Markdown wraps lines, so compare on a whitespace-normalised copy.
    let flat = readme.split_whitespace().collect::<Vec<_>>().join(" ");
    for expected in [
        "line counts",
        "not analyzed",
        "no code-versus-comment split",
    ] {
        assert!(flat.contains(expected), "README must document {expected}");
    }
}

#[test]
fn test_readme_documents_both_install_paths() {
    let readme = repo_file("README.md");
    for expected in [
        "install.sh",
        "install.ps1",
        "cargo install snipe-code",
        "no Rust toolchain",
        "Rust toolchain",
        "checksum",
    ] {
        assert!(readme.contains(expected), "README must document {expected}");
    }
    assert!(
        readme.contains("README.id.md"),
        "the Indonesian README must stay linked from the English one"
    );
    assert!(
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("README.id.md"))
            .expect("read README.id.md")
            .contains("README.md"),
        "the Indonesian README must link back to the English one"
    );
}

// --- installers never pipe into a shell ------------------------------------

#[test]
fn test_installers_never_pipe_anything_into_a_shell() {
    for (file, forbidden) in [
        (
            "install.sh",
            vec!["| sh", "|sh", "| bash", "|bash", "sh -c", "eval "],
        ),
        (
            "install.ps1",
            vec!["Invoke-Expression", "iex ", "Invoke-Expression ("],
        ),
    ] {
        let script = repo_file(file);
        for pattern in forbidden {
            assert!(
                !script.contains(pattern),
                "{file} must not pipe into a shell ({pattern})"
            );
        }
    }
}

// --- the container image carries the released binary ----------------------

#[test]
fn test_release_publishes_an_image_built_from_the_linux_archive() {
    let workflow = repo_file(".github/workflows/release.yml");
    let image_job = workflow
        .split("  image:")
        .nth(1)
        .expect("the release workflow must have an image job");

    for expected in [
        "release-x86_64-unknown-linux-gnu",
        "packages: write",
        "docker build",
        "docker push",
        "ghcr.io/",
        "docker login ghcr.io",
    ] {
        assert!(
            image_job.contains(expected),
            "the image job must {expected}:\n{image_job}"
        );
    }
    assert!(
        image_job.contains("docker run --rm"),
        "the image must be run before it is pushed, so a broken image never publishes:\n{image_job}"
    );

    let dockerfile = repo_file("Dockerfile");
    assert!(
        dockerfile.contains("COPY snipe-code /usr/local/bin/snipe-code"),
        "the image must carry the released binary:\n{dockerfile}"
    );
    assert!(
        dockerfile.contains("ENTRYPOINT [\"snipe-code\"]"),
        "the image must run the tool itself:\n{dockerfile}"
    );
}

#[test]
fn test_no_workflow_step_ends_with_a_negated_command() {
    for file in [".github/workflows/release.yml", ".github/workflows/ci.yml"] {
        let workflow = repo_file(file);
        for line in workflow.lines().map(str::trim) {
            assert!(
                !line.starts_with("! "),
                "{file} ends a step with a negated command, whose exit status inverts the \
                 step result: {line}"
            );
        }
    }

    let workflow = repo_file(".github/workflows/release.yml");
    assert!(
        workflow.contains("must not carry build artifacts"),
        "the package step must fail loudly when the archive stage carries build artifacts"
    );
}
