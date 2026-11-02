//! EmuCleaner core: all logic lives here so it can be tested without the UI.
//! Safety rules: files are never deleted directly. They are moved to a quarantine
//! folder (restorable) after passing the checks in `safety`.
pub mod boost;
pub mod history;
pub mod junk;
pub mod netcheck;
pub mod paths;
pub mod quarantine;
pub mod safety;
pub mod scan;
pub mod settings;
pub mod startup;
pub mod sys;
pub mod util;
