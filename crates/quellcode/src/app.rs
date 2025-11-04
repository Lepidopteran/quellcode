use std::sync::Mutex;

use color_eyre::eyre::Result;
use log::info;
use tauri::State;

use crate::{generator::GeneratorOptions, AppState};

#[tauri::command]
pub async fn generate_code(
    state: State<'_, Mutex<AppState>>,
    code: String,
    generator_name: String,
    syntax_name: String,
    theme_name: String,
    options: GeneratorOptions,
) -> Result<String, String> {
    info!("Generating code with generator {}", generator_name);

    let (syntax_set, syntect_themes, generators, context) = {
        let state = state.lock().expect("Failed to lock state");
        (
            state.syntect_syntaxes.clone(),
            state.syntect_themes.themes.clone(),
            state.generators.clone(),
            state.generator_context.clone(),
        )
    };

    let syntax = syntax_set.find_syntax_by_name(&syntax_name).cloned();
    let theme = syntect_themes.get(&theme_name).cloned();
    let generator = generators
        .iter()
        .find_map(|(info, gen)| (info.name() == generator_name).then_some(gen))
        .cloned();

    if let (Some(syntax), Some(theme), Some(generator)) = (syntax, theme, generator) {
        let result = tokio::task::spawn_blocking(move || {
            generator
                .generate_code(&code, &theme, &syntax, &syntax_set, &options, &context)
                .map_err(|err| err.to_string())
        })
        .await
        .map_err(|err| err.to_string())?;

        Ok(result?)
    } else {
        Err("Failed to find generator, syntax, or theme".to_string())
    }
}
