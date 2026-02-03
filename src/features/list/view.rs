use crate::core::types::Profile;
use console::{StyledObject, style};
use tabled::{Table, Tabled, settings::Style};

#[derive(Tabled)]
struct ProfileRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "REGION")]
    region: String,
    #[tabled(rename = "TAGS")]
    tags: String,
    #[tabled(rename = "ALIASES")]
    aliases: String,
    #[tabled(rename = "NOTE")]
    note: String,
}

#[derive(Tabled)]
struct CompactProfileRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "ALIASES")]
    aliases: String,
}

use crate::core::sso::{SsoService, SsoStatus};

pub struct ListView {
    short: bool,
    regex: Option<regex::Regex>,
    hl_config: HighlightConfig,
    sso_service: SsoService,
}

struct HighlightConfig {
    name: bool,
    aliases: bool,
    tags: bool,
    note: bool,
}

impl ListView {
    pub fn new(short: bool, query: &str, sso_service: SsoService) -> Self {
        let (regex, hl_config) = Self::parse_query(query);
        Self {
            short,
            regex,
            hl_config,
            sso_service,
        }
    }

    fn parse_query(query: &str) -> (Option<regex::Regex>, HighlightConfig) {
        if query.is_empty() {
            return (
                None,
                HighlightConfig {
                    name: false,
                    aliases: false,
                    tags: false,
                    note: false,
                },
            );
        }

        let query_lower = query.to_lowercase();
        let (pattern, config) = if let Some(val) = query_lower.strip_prefix("tag:") {
            (
                regex::escape(val),
                HighlightConfig {
                    name: false,
                    aliases: false,
                    tags: true,
                    note: false,
                },
            )
        } else if let Some(val) = query_lower.strip_prefix("alias:") {
            (
                regex::escape(val),
                HighlightConfig {
                    name: false,
                    aliases: true,
                    tags: false,
                    note: false,
                },
            )
        } else if let Some(val) = query_lower.strip_prefix("name:") {
            (
                regex::escape(val),
                HighlightConfig {
                    name: true,
                    aliases: false,
                    tags: false,
                    note: false,
                },
            )
        } else {
            (
                regex::escape(query),
                HighlightConfig {
                    name: true,
                    aliases: true,
                    tags: true,
                    note: true,
                },
            )
        };

        // Case insensitive regex
        let re = regex::Regex::new(&format!("(?i){}", pattern)).ok();
        (re, config)
    }

    pub fn render(&self, profiles: &[Profile]) {
        if profiles.is_empty() {
            println!("No profiles found.");
            return;
        }

        let table = if self.short {
            self.build_compact(profiles)
        } else {
            self.build_full(profiles)
        };
        println!("{}", table);
    }

    fn build_compact(&self, profiles: &[Profile]) -> String {
        let rows: Vec<CompactProfileRow> = profiles
            .iter()
            .map(|p| {
                let name_display = self.highlight(&p.name, self.hl_config.name);
                let name_styled = if self.hl_config.name && self.regex.is_some() {
                    name_display
                } else {
                    style(&p.name).bold().to_string()
                };

                let sso_status = self.sso_service.get_status(p);
                let sso_icon = match sso_status {
                    SsoStatus::Active => "🟢 ".to_string(),
                    SsoStatus::Expired => "🔴 ".to_string(),
                    SsoStatus::NotConfigured => "   ".to_string(),
                    SsoStatus::Unknown => "⚪ ".to_string(),
                };

                let name_final = format!("{}{}", sso_icon, name_styled);

                let aliases_str = if let Some(m) = &p.metadata {
                    self.highlight_list(&m.aliases, self.hl_config.aliases, |s| style(s).yellow())
                } else {
                    String::new()
                };

                CompactProfileRow {
                    name: name_final,
                    aliases: aliases_str,
                }
            })
            .collect();
        let mut t = Table::new(rows);
        t.with(Style::modern());
        t.to_string()
    }

    fn build_full(&self, profiles: &[Profile]) -> String {
        let rows: Vec<ProfileRow> = profiles
            .iter()
            .map(|p| {
                let name_display = self.highlight(&p.name, self.hl_config.name);
                let name_styled = if self.hl_config.name && self.regex.is_some() {
                    name_display
                } else {
                    style(&p.name).bold().to_string()
                };

                let sso_status = self.sso_service.get_status(p);
                let sso_icon = match sso_status {
                    SsoStatus::Active => "🟢 ".to_string(),
                    SsoStatus::Expired => "🔴 ".to_string(),
                    SsoStatus::NotConfigured => "   ".to_string(),
                    SsoStatus::Unknown => "⚪ ".to_string(),
                };

                let name_final = format!("{}{}", sso_icon, name_styled);

                let (tags, aliases, note) = if let Some(m) = &p.metadata {
                    (
                        self.highlight_list(&m.tags, self.hl_config.tags, |s| style(s).green()),
                        self.highlight_list(&m.aliases, self.hl_config.aliases, |s| {
                            style(s).yellow()
                        }),
                        {
                            let n_raw = m.note.as_deref().unwrap_or_default();
                            if self.hl_config.note && self.regex.is_some() {
                                self.highlight_note(n_raw)
                            } else {
                                style(n_raw).dim().to_string()
                            }
                        },
                    )
                } else {
                    (String::new(), String::new(), String::new())
                };

                ProfileRow {
                    name: name_final,
                    region: p.region.clone().unwrap_or_default(),
                    tags,
                    aliases,
                    note,
                }
            })
            .collect();
        let mut t = Table::new(rows);
        t.with(Style::modern());
        t.to_string()
    }

    // --- Highlighting Helpers ---

    fn highlight(&self, text: &str, should_hl: bool) -> String {
        if should_hl {
            if let Some(r) = &self.regex {
                r.replace_all(text, |caps: &regex::Captures| {
                    style(&caps[0]).red().bold().to_string()
                })
                .to_string()
            } else {
                text.to_string()
            }
        } else {
            text.to_string()
        }
    }

    fn highlight_list(
        &self,
        items: &[String],
        should_hl: bool,
        base_color: impl Fn(&str) -> StyledObject<&str>,
    ) -> String {
        items
            .iter()
            .map(|item| {
                let base_colored = base_color(item);
                if should_hl && self.regex.is_some() {
                    if let Some(r) = &self.regex {
                        let mut last_end = 0;
                        let mut res = String::new();
                        for m in r.find_iter(item) {
                            res.push_str(&base_color(&item[last_end..m.start()]).to_string());
                            res.push_str(&style(m.as_str()).red().bold().to_string());
                            last_end = m.end();
                        }
                        res.push_str(&base_color(&item[last_end..]).to_string());
                        res
                    } else {
                        base_colored.to_string()
                    }
                } else {
                    base_colored.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn highlight_note(&self, text: &str) -> String {
        if let Some(r) = &self.regex {
            let mut last_end = 0;
            let mut res = String::new();
            for m in r.find_iter(text) {
                res.push_str(&style(&text[last_end..m.start()]).dim().to_string());
                res.push_str(&style(m.as_str()).red().bold().to_string());
                last_end = m.end();
            }
            res.push_str(&style(&text[last_end..]).dim().to_string());
            res
        } else {
            style(text).dim().to_string()
        }
    }
}
