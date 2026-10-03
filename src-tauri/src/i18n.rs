//! Strings shown by the native side (the tray menu). Everything inside the
//! panel is translated in `src/i18n/` instead.

pub struct TrayStrings {
    pub open_panel: &'static str,
    pub quit: &'static str,
}

const EN: TrayStrings = TrayStrings {
    open_panel: "Open panel",
    quit: "Quit Farol",
};

const PT_BR: TrayStrings = TrayStrings {
    open_panel: "Abrir painel",
    quit: "Sair do Farol",
};

/// Picks the strings for a locale such as "pt-BR", "pt_BR" or "en-US".
pub fn tray_strings(locale: Option<&str>) -> &'static TrayStrings {
    let language = locale
        .and_then(|l| l.split(['-', '_']).next())
        .unwrap_or("en")
        .to_ascii_lowercase();
    match language.as_str() {
        "pt" => &PT_BR,
        _ => &EN,
    }
}

pub fn system_tray_strings() -> &'static TrayStrings {
    tray_strings(sys_locale::get_locale().as_deref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_by_primary_language() {
        assert_eq!(tray_strings(Some("pt-BR")).quit, PT_BR.quit);
        assert_eq!(tray_strings(Some("pt_PT")).quit, PT_BR.quit);
        assert_eq!(tray_strings(Some("en-US")).quit, EN.quit);
        assert_eq!(tray_strings(Some("ja-JP")).quit, EN.quit);
        assert_eq!(tray_strings(None).quit, EN.quit);
    }
}
