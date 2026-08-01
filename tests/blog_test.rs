use snipe_code::template::TemplateRegistry;
use snipe_code::generator::{self, GenerateConfig};

#[test]
fn test_blog_has_content_structure() {
    let template = TemplateRegistry::get("blog").unwrap();
    let folders = &template.structure.folders;

    assert!(folders.iter().any(|f| f.contains("posts")));
    assert!(folders.iter().any(|f| f.contains("categories")));
    assert!(folders.iter().any(|f| f.contains("tags")));
}

#[test]
fn test_blog_has_blog_components() {
    let template = TemplateRegistry::get("blog").unwrap();
    let folders = &template.structure.folders;

    assert!(folders.iter().any(|f| f.contains("post-card")));
    assert!(folders.iter().any(|f| f.contains("post-content")));
    assert!(folders.iter().any(|f| f.contains("author-page")));
}

#[test]
fn test_blog_has_sample_files() {
    let template = TemplateRegistry::get("blog").unwrap();
    assert!(template.structure.files.iter().any(|f| f.path.contains("hello-world.mdx")));
    assert!(template.structure.files.iter().any(|f| f.path.contains("rss.xml")));
}

#[test]
fn test_blog_backend_optional() {
    let template = TemplateRegistry::get("blog").unwrap();
    assert!(!template.structure.rules.require_backend);
}

#[test]
fn test_blog_generates_sample_post() {
    let config = GenerateConfig {
        project_name: "test-blog".to_string(),
        template_id: "blog".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config).unwrap();

    assert!(result.files.iter().any(|f| f.contains("hello-world.mdx")));

    let post_path = "test-blog/frontend/src/posts/hello-world.mdx";
    assert!(std::path::Path::new(post_path).exists());
    let content = std::fs::read_to_string(post_path).unwrap();
    assert!(content.contains("Hello World"));
    assert!(content.contains("title:"));

    let _ = std::fs::remove_dir_all("test-blog");
}

#[test]
fn test_blog_generates_rss_feed() {
    let config = GenerateConfig {
        project_name: "test-blog-rss".to_string(),
        template_id: "blog".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let result = generator::generate(&config).unwrap();

    assert!(result.files.iter().any(|f| f.contains("rss.xml")));

    let rss_path = "test-blog-rss/frontend/public/rss.xml";
    assert!(std::path::Path::new(rss_path).exists());
    let content = std::fs::read_to_string(rss_path).unwrap();
    assert!(content.contains("<rss"));
    assert!(content.contains("<channel>"));

    let _ = std::fs::remove_dir_all("test-blog-rss");
}

#[test]
fn test_blog_dry_run_shows_files() {
    let config = GenerateConfig {
        project_name: "blog-dry".to_string(),
        template_id: "blog".to_string(),
        frontend: "nextjs".to_string(),
        backend: "none".to_string(),
        database: "none".to_string(),
    };
    let output = generator::dry_run_preview(&config);

    assert!(output.contains("hello-world.mdx"));
    assert!(output.contains("rss.xml"));
    assert!(output.contains("posts"));
    assert!(output.contains("categories"));
}
