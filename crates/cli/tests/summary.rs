//! The lines of the scan summary, including what level 2 did per language.

#[cfg(test)]
mod cases {
    use codeskimr_application::tools::{ToolView, ToolsView};
    use codeskimr_cli::{
        LeftOutLine, LeftOutView, Level2Line, Level2Reason, ScanSummary, StageLine, WarningLine,
        scan_finished, scan_json,
    };
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
            pack_load_seconds: 0.5,
            stages: Vec::new(),
            level2,
            warnings: Vec::new(),
            left_out: LeftOutView::default(),
            tools: ToolsView::unchecked(),
        }
    }

    fn line(language: &str, stale: u64, reason: Option<Level2Reason>) -> Level2Line {
        Level2Line {
            language: language.to_owned(),
            files: 4,
            stale,
            outside_scan: 0,
            failed_workspaces: 0,
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
            line("python", 0, Some(Level2Reason::NoProjectFile)),
        ];
        let en = text("en", all.clone());
        assert!(contains(&en, &["Level 2", "csharp", "checked", "4 files"]));
        assert!(contains(&en, &["rust", "2 changed"]));
        assert!(contains(&en, &["go", "skipped", "confirmed"]));
        assert!(contains(&en, &["java", "skipped", "RES-502", "time"]));
        assert!(contains(&en, &["python", "skipped", "project file"]));
        let de = text("de", all);
        assert!(contains(
            &de,
            &["Stufe 2", "csharp", "geprüft", "4 Dateien"]
        ));
        assert!(contains(&de, &["go", "übersprungen", "bestätigt"]));
        assert!(contains(&de, &["java", "RES-502", "Zeitlimit"]));
        assert!(contains(&de, &["python", "übersprungen", "Projektdatei"]));
    }

    fn with_stage_times() -> ScanSummary {
        let stage = |stage: &str, seconds: f64| StageLine {
            stage: stage.to_owned(),
            seconds,
        };
        ScanSummary {
            stages: vec![stage("discover", 15.254_321), stage("resolve", 120.0)],
            ..summary(Vec::new())
        }
    }

    #[test]
    fn the_time_of_each_stage_is_a_line_in_both_languages_and_in_json() {
        let messages = Messages::load().unwrap();
        let en = scan_finished(&messages, "en", &with_stage_times()).unwrap();
        assert!(
            en.contains("Steps: discover 15.25 s, resolve 120 s."),
            "{en}"
        );
        let de = scan_finished(&messages, "de", &with_stage_times()).unwrap();
        assert!(
            de.contains("Schritte: discover 15.25 s, resolve 120 s."),
            "{de}"
        );
        let json = scan_json(&with_stage_times()).unwrap();
        assert!(
            json.contains(r#""stages":[{"stage":"discover","seconds":15.25},{"stage":"resolve","seconds":120.0}]"#),
            "{json}"
        );
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
    fn failed_workspaces_get_their_own_line_in_both_languages() {
        let failed = Level2Line {
            failed_workspaces: 2,
            ..line("rust", 0, None)
        };
        let en = text("en", vec![failed.clone()]);
        assert!(contains(&en, &["Level 2", "rust", "2 workspaces failed"]));
        let de = text("de", vec![failed.clone()]);
        assert!(contains(&de, &["Stufe 2", "rust", "2 Workspaces"]));
        assert_eq!(text("en", vec![line("rust", 0, None)]).len(), 1);
        let one = Level2Line {
            failed_workspaces: 1,
            ..line("rust", 0, None)
        };
        assert!(contains(
            &text("en", vec![one.clone()]),
            &["1 workspace failed and stays"]
        ));
        assert!(contains(
            &text("de", vec![one]),
            &["1 Workspace ist fehlgeschlagen"]
        ));
        let json = scan_json(&summary(vec![failed])).unwrap();
        assert!(json.contains("\"failed_workspaces\":2"), "{json}");
    }

    #[test]
    fn uses_in_skipped_folders_get_their_own_line_in_both_languages() {
        let outside = Level2Line {
            outside_scan: 80,
            ..line("rust", 0, None)
        };
        let en = text("en", vec![outside.clone()]);
        assert!(contains(&en, &["Level 2", "rust", "checked"]));
        assert!(contains(&en, &["skips", "not counted", "80"]));
        let de = text("de", vec![outside.clone()]);
        assert!(contains(&de, &["Stufe 2", "rust", "überspringt", "80"]));
        // Nothing outside the scan: no extra line.
        assert_eq!(text("en", vec![line("rust", 0, None)]).len(), 1);
        let json = scan_json(&summary(vec![outside])).unwrap();
        assert!(json.contains("\"outside_scan\":80"), "{json}");
    }

    #[test]
    fn json_reports_the_cpu_time_next_to_the_wall_time() {
        let json = scan_json(&summary(Vec::new())).unwrap();
        assert!(json.contains("\"cpu_seconds\":4.25"), "{json}");
    }

    #[test]
    fn json_reports_the_pack_load_time_apart_from_the_scan_time() {
        let json = scan_json(&summary(Vec::new())).unwrap();
        assert!(json.contains("\"pack_load_seconds\":0.5"), "{json}");
    }

    #[test]
    fn a_warning_is_one_line_with_its_count_in_both_languages() {
        let messages = Messages::load().unwrap();
        let mut with = summary(Vec::new());
        with.warnings = vec![
            WarningLine {
                code: "RES-301".to_owned(),
                count: 12,
            },
            WarningLine {
                code: "PRS-101".to_owned(),
                count: 3,
            },
        ];
        let en = scan_finished(&messages, "en", &with).unwrap();
        assert!(en.contains("RES-301: 12 places"), "{en}");
        assert!(
            en.contains("PRS-101: 3 files could not be analysed"),
            "{en}"
        );
        let de = scan_finished(&messages, "de", &with).unwrap();
        assert!(de.contains("RES-301: 12 Stellen"), "{de}");
        assert!(
            de.contains("PRS-101: 3 Dateien konnten nicht analysiert"),
            "{de}"
        );
        assert!(scan_json(&with).unwrap().contains("\"RES-301\""));
    }

    /// A summary with one file missing, one kept as the previous scan saw it, and three more.
    fn with_files_left_out() -> ScanSummary {
        let line = |path: &str, code: &str, kept| LeftOutLine {
            path: path.to_owned(),
            code: code.to_owned(),
            kept,
        };
        let mut with = summary(Vec::new());
        with.warnings = vec![WarningLine {
            code: "PRS-101".to_owned(),
            count: 1,
        }];
        with.left_out = LeftOutView {
            files: vec![
                line("src/alt.cs", "PRS-101", false),
                line("src/gross.cs", "PRS-501", true),
            ],
            more: 3,
        };
        with
    }

    #[test]
    fn the_files_left_out_are_named_and_the_others_counted_in_both_languages() {
        let messages = Messages::load().unwrap();
        let with = with_files_left_out();
        let en = scan_finished(&messages, "en", &with).unwrap();
        for part in [
            "PRS-101: one file could not be analysed",
            "  src/alt.cs (PRS-101): missing in the result",
            "  src/gross.cs (PRS-501): shown as the previous scan saw it",
            "  and 3 more files",
        ] {
            assert!(en.contains(part), "{part}: {en}");
        }
        let de = scan_finished(&messages, "de", &with).unwrap();
        for part in [
            "PRS-101: Eine Datei konnte nicht analysiert",
            "  src/alt.cs (PRS-101): fehlt im Ergebnis",
            "  src/gross.cs (PRS-501): Stand des vorigen Scans bleibt",
            "  und 3 weitere Dateien",
        ] {
            assert!(de.contains(part), "{part}: {de}");
        }
        let json = scan_json(&with).unwrap();
        assert!(json.contains("\"path\":\"src/alt.cs\""), "{json}");
        assert!(json.contains("\"more\":3"), "{json}");
    }

    fn crashed(found: Option<&str>) -> ScanSummary {
        let mut with = summary(Vec::new());
        with.tools = ToolsView {
            checked_at: Some(1),
            languages: Vec::new(),
            tools: vec![ToolView {
                tool: "scip-dotnet".to_owned(),
                role: "indexer",
                state: "not_runnable",
                languages: vec!["csharp".to_owned()],
                found: found.map(str::to_owned),
                required: None,
                path: None,
                purpose_key: "tool-scip-dotnet-purpose".to_owned(),
                install_key: "tool-scip-dotnet-install".to_owned(),
            }],
        };
        with
    }

    #[test]
    fn a_tool_that_does_not_start_is_named_with_its_error_in_both_languages() {
        let messages = Messages::load().unwrap();
        let with = crashed(Some("SyntaxError: boom"));
        let en = scan_finished(&messages, "en", &with).unwrap();
        assert!(en.contains("does not start"), "{en}");
        assert!(en.contains("SyntaxError: boom"), "{en}");
        let de = scan_finished(&messages, "de", &with).unwrap();
        assert!(de.contains("startet aber nicht"), "{de}");
        assert!(de.contains("SyntaxError: boom"), "{de}");
    }

    #[test]
    fn a_tool_that_does_not_start_and_said_nothing_has_no_dangling_colon_or_placeholder() {
        let messages = Messages::load().unwrap();
        let en = scan_finished(&messages, "en", &crashed(None)).unwrap();
        assert!(en.contains("it printed no error"), "{en}");
        assert!(!en.contains("– ?"), "{en}");
    }
}
