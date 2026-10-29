use crate::config::{Application, ApplicationSection, Config};
use crate::domain::{DetectedStack, EnabledWhen, Lint, LintDefinition, Result, Tool};
use std::fmt::Display;

pub struct Checkstyle;

const BINARY: &str = "checkstyle";
const CONFIG_ARG: &str = "-c /google_checks.xml";

impl Tool for Checkstyle {
    fn enabled_when(&self) -> EnabledWhen {
        EnabledWhen::Always
    }

    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application> {
        Some(apps.checkstyle.as_ref()?)
    }
}

impl Display for Checkstyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Checkstyle")
    }
}

impl Lint for Checkstyle {
    fn lint_commands(
        &self,
        _stack: &DetectedStack,
        _config: &Config,
    ) -> Result<Option<LintDefinition>> {
        if which::which(BINARY).is_err() {
            eprintln!(
                "checkstyle not found on PATH - skipping. Install with: brew install checkstyle",
            );
            return Ok(None);
        }
        Ok(Some(conc::shell_executable(format!(
            "{BINARY} {CONFIG_ARG} ."
        ))))
    }
}
