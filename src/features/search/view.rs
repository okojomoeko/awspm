use crate::core::error::AppError;
use crate::core::interaction::InteractionHandler;
use crate::core::policy::ProfileSafetyPolicy;
use crate::core::types::Profile;
use crate::features::search::SearchTarget;
use console::style;
use std::sync::Arc;

#[cfg(unix)]
use skim::prelude::*;
#[cfg(unix)]
use std::io::Cursor;

#[cfg(windows)]
use dialoguer::{FuzzySelect, theme::ColorfulTheme};

pub struct SearchView {
    policy: ProfileSafetyPolicy,
    handler: Arc<dyn InteractionHandler>,
}

impl SearchView {
    pub fn new(handler: Arc<dyn InteractionHandler>, policy: ProfileSafetyPolicy) -> Self {
        Self { policy, handler }
    }

    pub fn prompt_search(
        &self,
        profiles: &[Profile],
        target: SearchTarget,
        query: Option<String>,
    ) -> Result<Option<String>, AppError> {
        if profiles.is_empty() {
            println!("No profiles found.");
            return Ok(None);
        }

        // Calculate max widths for nice formatting
        let max_name_len = profiles
            .iter()
            .map(|p| p.name.len())
            .max()
            .unwrap_or(0)
            .max(4);
        let max_region_len = profiles
            .iter()
            .map(|p| p.region.as_deref().unwrap_or("").len())
            .max()
            .unwrap_or(0)
            .max(6);

        // Prepare display strings
        let display_items: Vec<String> = profiles
            .iter()
            .map(|p| {
                let region = p.region.as_deref().unwrap_or("");
                let tags = p
                    .metadata
                    .as_ref()
                    .map_or(String::new(), |m| m.tags.join(", "));
                let aliases = p
                    .metadata
                    .as_ref()
                    .map_or(String::new(), |m| m.aliases.join(", "));
                let note = p
                    .metadata
                    .as_ref()
                    .and_then(|m| m.note.clone())
                    .unwrap_or_default();

                match target {
                    SearchTarget::All => format!(
                        "{:<width$} | {:<region_w$} | {:<tags$} | {:<aliases$} | {}",
                        p.name,
                        region,
                        tags,
                        aliases,
                        note,
                        width = max_name_len,
                        region_w = max_region_len,
                        tags = 4,
                        aliases = 7
                    ),
                    SearchTarget::Tags => {
                        format!("{:<width$} | {}", p.name, tags, width = max_name_len)
                    }
                    SearchTarget::Aliases => {
                        format!("{:<width$} | {}", p.name, aliases, width = max_name_len)
                    }
                    SearchTarget::Name => p.name.clone(),
                }
            })
            .collect();

        // --- UNIX IMPLEMENTATION (Skim) ---
        #[cfg(unix)]
        {
            let input_data = display_items.join("\n");

            let mut options_builder = SkimOptionsBuilder::default();
            options_builder.height("50%".to_string());
            options_builder.multi(false);
            if let Some(q) = &query {
                options_builder.query(q.to_string());
            }

            let options = options_builder.build().map_err(|e| anyhow::anyhow!(e))?;
            let item_reader = SkimItemReader::default();
            let items = item_reader.of_bufread(Cursor::new(input_data));

            let selected_items = Skim::run_with(options, Some(items))
                .map(|out| out.selected_items)
                .unwrap_or_default();

            if let Some(item) = selected_items.first() {
                let output = item.output();
                let selected_profile_name =
                    output.split('|').next().unwrap_or("").trim().to_string();
                return self.finalize_selection(profiles, selected_profile_name);
            }
        }

        // --- WINDOWS IMPLEMENTATION (Dialoguer) ---
        #[cfg(windows)]
        {
            let theme = ColorfulTheme::default();
            let mut selection = FuzzySelect::with_theme(&theme);
            selection = selection.items(&display_items).default(0);

            if let Some(q) = query {
                selection = selection.with_initial_text(q);
            }

            if let Ok(index) = selection.interact_opt() {
                if let Some(idx) = index {
                    // dialoguer returns index, so we can get the profile directly
                    let selected_profile_name = profiles[idx].name.clone();
                    return self.finalize_selection(profiles, selected_profile_name);
                }
            }
        }

        Ok(None)
    }

    fn finalize_selection(
        &self,
        profiles: &[Profile],
        selected_profile_name: String,
    ) -> Result<Option<String>, AppError> {
        // Safety Check
        if let Some(profile) = profiles.iter().find(|p| p.name == selected_profile_name) {
            let tags = profile
                .metadata
                .as_ref()
                .map(|m| m.tags.clone())
                .unwrap_or_default();
            if self
                .policy
                .is_sensitive_profile(&selected_profile_name, &tags)
                && !self
                    .handler
                    .confirm_sensitive_action(&selected_profile_name)
            {
                eprintln!("{}", style("Aborted.").red());
                return Ok(None);
            }
        }
        Ok(Some(selected_profile_name))
    }
}
