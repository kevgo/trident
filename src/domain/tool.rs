use crate::config::{Application, ApplicationSection};
use crate::domain::EnabledWhen;
use std::fmt::Display;

/// a tool (lint or fix) that Trident can run
pub trait Tool: Display {
    /// provides the application-specific section in the config file
    fn config_section<'a>(&self, apps: &'a ApplicationSection) -> Option<&'a dyn Application>;

    fn enabled_when(&self) -> EnabledWhen;
}
