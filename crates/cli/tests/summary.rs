//! The lines of the scan summary, including what level 2 did per language.

#[cfg(test)]
mod cases {
    use codeskimr_application::tools::ToolsView;
    use codeskimr_cli::{Level2Line, Level2Reason, ScanSummary, scan_finished, scan_json};
    use codeskimr_i18n::Messages;

    fn summary(level2: Vec<Level2Line>) -> ScanSummary {
        ScanSummary {
            project: "demo".to_owned(),
            snapshot: 3,
            files: 10,
            symbols: 20,
            references: 30,
            seconds: 1.5,
            cpu_seconds: 4.25,
            level2,
            tools: ToolsView::unchecked(),
        }
    }

    fn line(language: &str, stale: u64, reason: Option<Level2Reason>) -> Level2Line {
        Level2Line {
            language: language.to_owned(),
            files: 4,
            stale,
            reason,
        }
    }

    fn text(language: &str, level2: Vec<Level2Line>) -> Vec<String> {
        let messages = Messages::load().unwrap();
        let text = scan_finished(&messages, language, &summary(level2)).unwrap();
        text.lines().skip(2).map(str::to_owned).collect()
    }

    fn contains(lines: &[String], parts: &[&str]) -> bool {
        lines
            .iter()
            .any(|line| parts.iter().all(|part| line.contains(part)))
    }

    #[test]
    fn without_level_two_the_summary_stays_two_lines() {
        assert_eq!(text("en", Vec::new()), Vec::<String>::new());
    }

    #[test]
    fn every_language_gets_a_line_in_both_languages() {
        let all = vec![
            line("csharp", 0, None),
            line("rust", 2, None),
            line("go", 0, Some(Level2Reason::NotConfirmed)),
            line("java", 0, Some(Level2Reason::Failed("RES-502".to_owned()))),
        ];
        let en = text("en", all.clone());
        assert!(contains(&en, &["Level 2", "csharp", "checked", "4 files"]));
        assert!(contains(&en, &["rust", "2 changed"]));
        assert!(contains(&en, &["go", "skipped", "confirmed"]));
        assert!(contains(&en, &["java", "skipped", "RES-502"]));
        let de = text("de", all);
        assert!(contains(
            &de,
            &["Stufe 2", "csharp", "geprüft", "4 Dateien"]
        ));
        assert!(contains(&de, &["go", "übersprungen", "bestätigt"]));
        assert!(contains(&de, &["java", "RES-502"]));
    }

    #[test]
    fn json_carries_the_reason_of_a_skipped_language() {
        let json = scan_json(&summary(vec![line(
            "go",
            0,
            Some(Level2Reason::NotConfirmed),
        )]))
        .unwrap();
        assert!(json.contains("\"not_confirmed\""), "{json}");
    }

    #[test]
    fn json_reports_the_cpu_time_next_to_the_wall_time() {
        let json = scan_json(&summary(Vec::new())).unwrap();
        assert!(json.contains("\"cpu_seconds\":4.25"), "{json}");
    }
}
