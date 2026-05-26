fn main() -> anyhow::Result<()> {
    let api_doc = api::openapi::build_openapi();
    println!("{}", serde_json::to_string_pretty(&api_doc)?);
    Ok(())
}
