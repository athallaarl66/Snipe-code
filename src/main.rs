use snipe_code::template::TemplateRegistry;

fn main() {
    let templates = TemplateRegistry::load_all();
    println!("Loaded {} templates", templates.len());
}
