use lazy_regex::regex_captures;
use poise::ChoiceParameter;

#[derive(ChoiceParameter)]
pub enum JamType {
    #[name = "Itch.io jam"]
    Itch,
    #[name = "Ludum Dare"]
    LudumDare,
}

impl JamType {
    pub fn jam_link_example(&self) -> &'static str {
        use JamType::*;

        match self {
            Itch => "https://itch.io/jam/example-jam",
            LudumDare => "https://ldjam.com/events/ludum-dare/123456",
        }
    }

    pub fn normalize_jam_link(&self, link: &str) -> Option<String> {
        use JamType::*;

        match self {
            Itch => {
                let (_whole, link_normalized) =
                    regex_captures!(r#"^(https://itch\.io/jam/[a-z0-9_-]+)/?$"#, link)?;

                Some(link_normalized.to_owned())
            }

            LudumDare => {
                let (_whole, link_normalized) =
                    regex_captures!(r#"^(https://ldjam.com/events/ludum-dare/[0-9]+)/?$"#, link)?;

                Some(link_normalized.to_owned())
            }
        }
    }

    pub fn jam_entry_link_example(&self, jam_link: &str) -> String {
        use JamType::*;

        match self {
            Itch => jam_link.to_owned() + "/rate/123456",
            LudumDare => jam_link.to_owned() + "/example-game",
        }
    }

    pub fn normalize_jam_entry_link(&self, jam_link: &str, entry_link: &str) -> Option<String> {
        use JamType::*;

        match self {
            Itch => {
                let tail = entry_link.strip_prefix(jam_link)?;
                let (_whole, id) = regex_captures!(r#"/rate/([0-9]+)/?"#, tail)?;

                Some(format!("{jam_link}/rate/{id}"))
            }

            LudumDare => {
                let tail = entry_link.strip_prefix(jam_link)?;
                let (_whole, slug) = regex_captures!(r#"/([a-z0-9-]+)/?"#, tail)?;

                // Some slugs coincide with LD jam pages
                match slug {
                    "results" | "games" | "theme" | "stats" => None,

                    slug if slug.is_empty() => None,

                    slug => Some(format!("{jam_link}/{slug}")),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::jam_types::JamType;

    #[test]
    fn itch_jam_link_example_is_valid() {
        assert!(JamType::Itch
            .normalize_jam_link(JamType::Itch.jam_link_example())
            .is_some());
    }

    #[test]
    fn itch_jam_link_valid_without_trailing_slash() {
        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/bevy-jam-2")
            .is_some());
    }

    #[test]
    fn itch_jam_link_valid_with_trailing_slash() {
        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/bevy_jam_2/")
            .is_some());
    }

    #[test]
    fn itch_jam_link_invalid() {
        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/bevy-jam-2/rate/1675016")
            .is_none());
    }

    #[test]
    fn itch_jam_entry_link_invalid() {
        assert!(JamType::Itch
            .normalize_jam_link("https://redteapot.itch.io/one-clicker")
            .is_none());
    }

    #[test]
    fn itch_jam_entry_example_is_valid() {
        let jam = JamType::Itch
            .normalize_jam_link(JamType::Itch.jam_link_example())
            .unwrap();

        let entry = JamType::Itch.jam_entry_link_example(&jam);
        assert!(JamType::Itch
            .normalize_jam_entry_link(&jam, &entry)
            .is_some());
    }

    #[test]
    fn itch_jam_entry_link_valid_without_trailing_slash() {
        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/bevy-jam-2",
                "https://itch.io/jam/bevy-jam-2/rate/1675016"
            )
            .is_some());
    }

    #[test]
    fn itch_jam_entry_link_valid_with_trailing_slash() {
        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/bevy-jam-2",
                "https://itch.io/jam/bevy-jam-2/rate/1675016/"
            )
            .is_some());
    }

    #[test]
    fn itch_jam_pages_are_not_entries() {
        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/foo_bar_1234567890/entries")
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/foo_bar_1234567890/entries/")
            .is_none());

        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/foo_bar_1234567890/results")
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/foo_bar_1234567890/results/")
            .is_none());

        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/foo_bar_1234567890/community")
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/foo_bar_1234567890/community/")
            .is_none());

        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/foo_bar_1234567890/screenshots")
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/foo_bar_1234567890/screenshots/")
            .is_none());

        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/foo_bar-123456-7890/feed")
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/foo_bar-123456-7890/feed/")
            .is_none());
    }

    #[test]
    fn itch_jam_entry_pages_are_not_entries() {
        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar_1234567890",
                "https://itch.io/jam/foo_bar_1234567890"
            )
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar_1234567890",
                "https://itch.io/jam/foo_bar_1234567890/"
            )
            .is_none());

        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar_1234567890",
                "https://itch.io/jam/foo_bar_1234567890/entries"
            )
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar_1234567890",
                "https://itch.io/jam/foo_bar_1234567890/entries/"
            )
            .is_none());

        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar_1234567890",
                "https://itch.io/jam/foo_bar_1234567890/results"
            )
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar_1234567890",
                "https://itch.io/jam/foo_bar_1234567890/results/"
            )
            .is_none());

        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar_1234567890",
                "https://itch.io/jam/foo_bar_1234567890/community"
            )
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar_1234567890",
                "https://itch.io/jam/foo_bar_1234567890/community/"
            )
            .is_none());

        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar_1234567890",
                "https://itch.io/jam/foo_bar_1234567890/screenshots"
            )
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar_1234567890",
                "https://itch.io/jam/foo_bar_1234567890/screenshots/"
            )
            .is_none());

        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar-123456-7890",
                "https://itch.io/jam/foo_bar-123456-7890/feed"
            )
            .is_none());
        assert!(JamType::Itch
            .normalize_jam_entry_link(
                "https://itch.io/jam/foo_bar-123456-7890",
                "https://itch.io/jam/foo_bar-123456-7890/feed/"
            )
            .is_none());
    }

    #[test]
    fn ludum_dare_jam_link_example_is_valid() {
        assert!(JamType::LudumDare
            .normalize_jam_link(JamType::LudumDare.jam_link_example())
            .is_some());
    }

    #[test]
    fn ludum_dare_jam_link_valid_without_trailing_slash() {
        assert!(JamType::LudumDare
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/49")
            .is_some());
    }

    #[test]
    fn ludum_dare_jam_link_valid_with_trailing_slash() {
        assert!(JamType::LudumDare
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/49/")
            .is_some());
    }

    #[test]
    fn ludum_dare_jam_entry_example_is_valid() {
        let jam = JamType::LudumDare
            .normalize_jam_link(JamType::LudumDare.jam_link_example())
            .unwrap();

        let entry = JamType::LudumDare.jam_entry_link_example(&jam);
        assert!(JamType::LudumDare
            .normalize_jam_entry_link(&jam, &entry)
            .is_some());
    }

    #[test]
    fn ludum_dare_jam_entry_link_valid_without_trailing_slash() {
        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/49",
                "https://ldjam.com/events/ludum-dare/49/unstable98-exe"
            )
            .is_some());
    }

    #[test]
    fn ludum_dare_jam_entry_link_valid_with_trailing_slash() {
        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/49",
                "https://ldjam.com/events/ludum-dare/49/unstable98-exe/"
            )
            .is_some());
    }

    #[test]
    fn ludum_dare_jam_link_invalid() {
        assert!(JamType::Itch
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/49/unstable98-exe")
            .is_none());
    }

    #[test]
    fn ludum_dare_jam_entry_link_invalid() {
        assert!(JamType::Itch
            .normalize_jam_link("https://itch.io/jam/bevy-jam-2/rate/1675016")
            .is_none());
    }

    #[test]
    fn ludum_dare_jam_pages_are_not_entries() {
        assert!(JamType::LudumDare
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/5/results")
            .is_none());
        assert!(JamType::LudumDare
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/6/results/")
            .is_none());

        assert!(JamType::LudumDare
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/78/games")
            .is_none());
        assert!(JamType::LudumDare
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/90/games/")
            .is_none());

        assert!(JamType::LudumDare
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/500/theme")
            .is_none());
        assert!(JamType::LudumDare
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/512/theme/")
            .is_none());

        assert!(JamType::LudumDare
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/49/stats")
            .is_none());
        assert!(JamType::LudumDare
            .normalize_jam_link("https://ldjam.com/events/ludum-dare/49/stats/")
            .is_none());
    }

    #[test]
    fn ludum_dare_jam_entry_pages_are_not_entries() {
        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/5",
                "https://ldjam.com/events/ludum-dare/5/results"
            )
            .is_none());
        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/6",
                "https://ldjam.com/events/ludum-dare/6/results/"
            )
            .is_none());

        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/5",
                "https://ldjam.com/events/ludum-dare/5/results"
            )
            .is_none());
        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/6",
                "https://ldjam.com/events/ludum-dare/6/results/"
            )
            .is_none());

        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/78",
                "https://ldjam.com/events/ludum-dare/78/games"
            )
            .is_none());
        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/90",
                "https://ldjam.com/events/ludum-dare/90/games/"
            )
            .is_none());

        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/500",
                "https://ldjam.com/events/ludum-dare/500/theme"
            )
            .is_none());
        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/512",
                "https://ldjam.com/events/ludum-dare/512/theme/"
            )
            .is_none());

        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/49",
                "https://ldjam.com/events/ludum-dare/49/stats"
            )
            .is_none());
        assert!(JamType::LudumDare
            .normalize_jam_entry_link(
                "https://ldjam.com/events/ludum-dare/49",
                "https://ldjam.com/events/ludum-dare/49/stats/"
            )
            .is_none());
    }
}
