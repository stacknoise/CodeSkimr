//! The layout of an abapGit repository: the configuration, the object of a file name and the
//! summary for the project page (ADR 0113).

#[cfg(test)]
mod tests {
    use codeskimr_application::abapgit::{
        AbapGitConfig, FolderLogic, object_of, parse_config, summarize,
    };
    use codeskimr_application::ports::ListedFile;
    use codeskimr_domain::{LanguageId, RelativePath};

    fn file(path: &str) -> ListedFile {
        let code = std::path::Path::new(path)
            .extension()
            .is_some_and(|extension| extension == "abap");
        ListedFile {
            path: RelativePath::new(path).unwrap(),
            language: code.then(|| LanguageId::new("abap").unwrap()),
        }
    }

    fn files(paths: &[&str]) -> Vec<ListedFile> {
        paths.iter().map(|path| file(path)).collect()
    }

    #[test]
    fn the_configuration_is_read_from_the_tags() {
        let text = "<DATA><STARTING_FOLDER>/code/</STARTING_FOLDER>\n<FOLDER_LOGIC>FULL</FOLDER_LOGIC></DATA>";
        let config = parse_config(text);
        assert_eq!(config.starting_folder, "code");
        assert_eq!(config.folder_logic, FolderLogic::Full);
    }

    #[test]
    fn what_the_file_does_not_say_is_the_default() {
        assert_eq!(parse_config("<DATA/>"), AbapGitConfig::default());
        assert_eq!(parse_config("").starting_folder, "src");
        let root = parse_config("<STARTING_FOLDER>/</STARTING_FOLDER>");
        assert_eq!(root.starting_folder, "");
    }

    #[test]
    fn a_file_name_names_its_object() {
        let class = object_of("zcl_a.clas.testclasses.abap").unwrap();
        assert_eq!(
            (class.object.name.as_str(), class.object.kind.as_str()),
            ("zcl_a", "CLAS")
        );
        assert!(class.is_source);
        let metadata = object_of("zcl_a.clas.xml").unwrap();
        assert!(!metadata.is_source);
        assert_eq!(metadata.object, class.object);
        assert_eq!(
            object_of("#ns#name.prog.abap").unwrap().object.name,
            "/ns/name"
        );
        assert!(object_of("zv.ddls.asddls").unwrap().is_source);
    }

    #[test]
    fn other_files_are_no_objects() {
        for name in [
            "package.devc.xml",
            "README.md",
            "zcl_a.abap",
            "zcl_a.clas.txt",
            ".abapgit.xml",
            "x",
        ] {
            assert_eq!(object_of(name), None, "{name}");
        }
    }

    #[test]
    fn the_summary_counts_objects_packages_and_source_files() {
        let listed = files(&[
            ".abapgit.xml",
            "README.md",
            "src/package.devc.xml",
            "src/zcl_a.clas.abap",
            "src/zcl_a.clas.testclasses.abap",
            "src/zcl_a.clas.xml",
            "src/zcl_b.clas.abap",
            "src/zif_a.intf.abap",
            "src/zt.tabl.xml",
            "src/sub/package.devc.xml",
            "src/sub/zprog.prog.abap",
            "other/zcl_c.clas.abap",
        ]);
        let summary = summarize(&listed, Some(&AbapGitConfig::default())).unwrap();
        assert!(summary.configured);
        assert_eq!(
            summary.objects,
            [
                ("CLAS".to_owned(), 2),
                ("INTF".to_owned(), 1),
                ("PROG".to_owned(), 1),
                ("TABL".to_owned(), 1)
            ]
        );
        assert_eq!(summary.packages, 2);
        assert_eq!(summary.source_files, 5);
    }

    #[test]
    fn a_repository_without_its_config_file_is_found_by_its_sources() {
        let listed = files(&["src/zcl_a.clas.abap", "src/zif_a.intf.abap"]);
        let summary = summarize(&listed, None).unwrap();
        assert!(!summary.configured);
        assert_eq!(summary.objects.len(), 2);
    }

    #[test]
    fn a_folder_without_abapgit_is_none() {
        assert_eq!(summarize(&files(&["src/main.rs", "README.md"]), None), None);
        assert_eq!(summarize(&files(&["lib/zcl_a.clas.abap"]), None), None);
        let empty = summarize(&files(&[".abapgit.xml"]), Some(&AbapGitConfig::default())).unwrap();
        assert!(empty.objects.is_empty());
    }

    #[test]
    fn the_project_folder_can_be_the_starting_folder() {
        let listed = files(&[".abapgit.xml", "zcl_a.clas.abap", "sub/zcl_b.clas.abap"]);
        let config = parse_config("<STARTING_FOLDER>/</STARTING_FOLDER>");
        let summary = summarize(&listed, Some(&config)).unwrap();
        assert_eq!(summary.objects, [("CLAS".to_owned(), 2)]);
        assert_eq!(summary.packages, 2);
    }
}
