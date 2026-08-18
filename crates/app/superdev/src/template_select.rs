//! template_select.rs — how init picks a project template. Flags answer in
//! advance (`--template <name>`, `--template none`, `--name`); a TTY with no
//! flag gets a prompt; a non-interactive run gets no template, so scripted
//! init keeps today's contract. The decision logic sits behind [`Prompter`]
//! so tests script the answers; the dialoguer adapter is terminal glue,
//! excluded from coverage.

use std::io::{self, IsTerminal};

use superdev_core::error::{Error, Result};
use superdev_core::templates::{self, Template, Tokens};

/// The `--template` help line. A test keeps it naming every shipped template.
pub const TEMPLATE_HELP: &str =
    "Project template to seed the repo from: rust-npm, or `none` (a TTY prompts when omitted)";

/// The questions the interactive path can ask.
pub trait Prompter {
    /// Pick one of `items`, returning its index; `default` is preselected.
    fn select(&self, prompt: &str, items: &[String], default: usize) -> Result<usize>;
    /// Free-text input, prefilled with an editable default.
    fn input(&self, prompt: &str, default: &str) -> Result<String>;
}

/// What init decided: the template to seed, with its token values.
#[derive(Debug)]
pub struct Selection {
    /// The shipped template to seed from.
    pub template: &'static Template,
    /// The substitution values, recorded in the manifest afterwards.
    pub tokens: Tokens,
}

/// Resolve the template decision from flags, TTY-ness and (interactively) the
/// prompter. `dir_name` is the derived project-name default.
pub fn choose(
    template_flag: Option<&str>,
    name_flag: Option<&str>,
    tty: bool,
    dir_name: &str,
    prompter: &dyn Prompter,
) -> Result<Option<Selection>> {
    let template = match template_flag {
        Some("none") => return Ok(None),
        Some(name) => templates::find(name).ok_or_else(|| Error::Manifest {
            message: format!("template must be one of: {} — or `none`", shipped_names()),
        })?,
        None if !tty => return Ok(None),
        None => {
            let mut items = vec!["none — start from the repo as it is".to_string()];
            items.extend(
                templates::shipped()
                    .iter()
                    .map(|t| format!("{} — {}", t.name, t.description)),
            );
            match prompter.select("Seed the repo from a project template?", &items, 0)? {
                0 => return Ok(None),
                picked => &templates::shipped()[picked - 1],
            }
        }
    };
    let name = match name_flag {
        Some(name) => name.to_string(),
        None if tty => prompter.input("Project name", dir_name)?,
        None => dir_name.to_string(),
    };
    Ok(Some(Selection {
        template,
        tokens: Tokens::for_name(&name),
    }))
}

fn shipped_names() -> String {
    templates::shipped()
        .iter()
        .map(|t| t.name)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Whether this run can prompt at all: both ends of the conversation must be
/// a terminal, or the answers would come from (or vanish into) a pipe.
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn is_tty() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

/// The real prompter. Thin dialoguer glue: everything decidable is decided in
/// [`choose`], so nothing here is worth a terminal-emulating test.
pub struct TerminalPrompter;

#[cfg_attr(coverage_nightly, coverage(off))]
impl Prompter for TerminalPrompter {
    fn select(&self, prompt: &str, items: &[String], default: usize) -> Result<usize> {
        dialoguer::Select::new()
            .with_prompt(prompt)
            .items(items)
            .default(default)
            .interact()
            .map_err(prompt_failed)
    }

    fn input(&self, prompt: &str, default: &str) -> Result<String> {
        dialoguer::Input::<String>::new()
            .with_prompt(prompt)
            .default(default.to_string())
            .interact_text()
            .map_err(prompt_failed)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn prompt_failed(e: dialoguer::Error) -> Error {
    Error::Manifest {
        message: format!("prompt failed: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scripted answers; panics if a question it was not given arrives.
    struct Fake {
        select: Option<usize>,
        input: Option<&'static str>,
    }

    impl Prompter for Fake {
        fn select(&self, _: &str, items: &[String], default: usize) -> Result<usize> {
            assert_eq!(default, 0, "none is the safe default");
            assert!(items[0].starts_with("none"), "{items:?}");
            Ok(self.select.expect("select was not scripted"))
        }
        fn input(&self, _: &str, default: &str) -> Result<String> {
            assert_eq!(default, "my-dir", "prefilled with the directory name");
            Ok(self.input.expect("input was not scripted").to_string())
        }
    }

    const NO_PROMPT: Fake = Fake {
        select: None,
        input: None,
    };

    #[test]
    fn flags_answer_in_advance_and_skip_the_prompt() {
        let sel = choose(
            Some("rust-npm"),
            Some("My Tool"),
            true,
            "my-dir",
            &NO_PROMPT,
        )
        .unwrap()
        .unwrap();
        assert_eq!(sel.template.name, "rust-npm");
        assert_eq!(sel.tokens.slug, "my-tool");
        assert!(
            choose(Some("none"), None, true, "my-dir", &NO_PROMPT)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn an_unknown_template_fails_with_the_shipped_list() {
        let err = choose(Some("flying"), None, false, "my-dir", &NO_PROMPT)
            .unwrap_err()
            .to_string();
        assert!(err.contains("rust-npm"), "{err}");
        assert!(err.contains("`none`"), "{err}");
    }

    #[test]
    fn no_tty_means_no_prompt_and_no_template() {
        assert!(
            choose(None, None, false, "my-dir", &NO_PROMPT)
                .unwrap()
                .is_none()
        );
        // The name flag alone changes nothing without a template.
        assert!(
            choose(None, Some("My Tool"), false, "my-dir", &NO_PROMPT)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_tty_prompts_for_template_and_name() {
        let fake = Fake {
            select: Some(1),
            input: Some("My Tool"),
        };
        let sel = choose(None, None, true, "my-dir", &fake).unwrap().unwrap();
        assert_eq!(sel.template.name, "rust-npm");
        assert_eq!(sel.tokens.name, "My Tool");

        let none = Fake {
            select: Some(0),
            input: None,
        };
        assert!(choose(None, None, true, "my-dir", &none).unwrap().is_none());
    }

    #[test]
    fn a_flagged_template_without_a_tty_derives_the_name() {
        let sel = choose(Some("rust-npm"), None, false, "my-dir", &NO_PROMPT)
            .unwrap()
            .unwrap();
        assert_eq!(sel.tokens.name, "my-dir");
    }

    #[test]
    fn the_help_line_names_every_shipped_template() {
        for template in templates::shipped() {
            assert!(
                TEMPLATE_HELP.contains(template.name),
                "--template help omits {}",
                template.name
            );
        }
    }
}
