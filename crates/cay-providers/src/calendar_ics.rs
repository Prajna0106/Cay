use crate::traits::{CalendarProvider, ProviderError};
use cay_core::ics::parse_ics_meetings;
use cay_core::models::CalendarMeeting;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// File-backed or local cache-backed `.ics` iCalendar provider for MVP (P0).
#[derive(Clone, Debug)]
pub struct IcsCalendarProvider {
    ics_path: Option<PathBuf>,
}

impl Default for IcsCalendarProvider {
    fn default() -> Self {
        let env_path = std::env::var_os("CAY_ICS_PATH").map(PathBuf::from);
        Self { ics_path: env_path }
    }
}

impl IcsCalendarProvider {
    pub fn new(ics_path: Option<PathBuf>) -> Self {
        Self { ics_path }
    }
}

impl CalendarProvider for IcsCalendarProvider {
    fn get_upcoming_meetings(
        &self,
        lookahead_minutes: u32,
    ) -> Result<Vec<CalendarMeeting>, ProviderError> {
        let Some(path) = &self.ics_path else {
            return Ok(Vec::new());
        };

        let content = std::fs::read_to_string(path).map_err(|e| {
            ProviderError::Unavailable(format!("Failed to read .ics file {}: {e}", path.display()))
        })?;

        let now_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        Ok(parse_ics_meetings(&content, now_secs, lookahead_minutes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ics_provider_without_path_returns_empty() {
        let provider = IcsCalendarProvider::new(None);
        let events = provider.get_upcoming_meetings(15).unwrap();
        assert!(events.is_empty());
    }
}
