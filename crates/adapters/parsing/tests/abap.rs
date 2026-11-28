//! The ABAP pack (ADR 0113): names are folded to lower case, an interface method is named after
//! the method, `WHEN OTHERS` is no decision and the grammar is compiled in.

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::{fs, time::Duration};

    use super::common::analyze_file;
    use codeskimr_application::{ids::IdSequence, ports::FileFacts};
    use codeskimr_domain::FolderPath;
    use codeskimr_parsing::{LanguagePacks, TreeSitterAnalyzer};

    const FILE: &str = "zcl_box.clas.abap";

    fn languages() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../languages")
    }

    fn analyze(source: &str) -> FileFacts {
        let folder = tempfile::tempdir().expect("folder");
        fs::write(folder.path().join(FILE), source).expect("file");
        let analyzer = TreeSitterAnalyzer::new(
            LanguagePacks::load(&languages()).expect("packs load"),
            Duration::from_secs(10),
        );
        analyze_file(
            &analyzer,
            &IdSequence::starting_at(1),
            &FolderPath::new(folder.path().to_string_lossy()).expect("root"),
            FILE,
            "abap",
        )
    }

    fn names(facts: &FileFacts) -> Vec<String> {
        facts
            .symbols
            .iter()
            .map(|symbol| symbol.qualified_name.to_string())
            .collect()
    }

    #[test]
    fn names_are_folded_to_lower_case_and_methods_belong_to_their_class() {
        let facts = analyze(
            "CLASS ZCL_Box DEFINITION PUBLIC.\n  PUBLIC SECTION.\n    METHODS Run.\nENDCLASS.\n\
             CLASS zcl_BOX IMPLEMENTATION.\n  METHOD RUN.\n    Helper( ).\n    PERFORM Do_It.\n  ENDMETHOD.\nENDCLASS.\n\
             FORM DO_IT.\nENDFORM.\n",
        );
        assert_eq!(
            names(&facts),
            ["zcl_box.clas.abap", "zcl_box", "zcl_box=>run", "do_it"]
        );
        let targets: Vec<&str> = facts
            .pending
            .iter()
            .map(|reference| reference.target.as_str())
            .collect();
        assert_eq!(targets, ["helper", "do_it"]);
    }

    #[test]
    fn an_interface_method_is_named_after_the_method() {
        let facts = analyze(
            "CLASS zcl_box DEFINITION PUBLIC.\n  PUBLIC SECTION.\n    INTERFACES zif_shape.\nENDCLASS.\n\
             CLASS zcl_box IMPLEMENTATION.\n  METHOD zif_shape~area.\n    rv = 1.\n  ENDMETHOD.\nENDCLASS.\n",
        );
        assert!(names(&facts).contains(&"zcl_box=>area".to_owned()));
    }

    #[test]
    fn only_the_abap_pack_shows_the_folder_as_the_namespace() {
        let packs = LanguagePacks::load(&languages()).unwrap();
        let from_folder: Vec<_> = packs
            .iter()
            .filter(|pack| pack.namespace_from_folder())
            .map(|pack| pack.id().as_str().to_owned())
            .collect();
        assert_eq!(from_folder, ["abap"]);
    }

    #[test]
    fn when_others_is_no_decision() {
        let facts = analyze(
            "FORM f.\n  CASE x.\n    WHEN 1.\n      a = 1.\n    WHEN 2 OR 3.\n      a = 2.\n    WHEN OTHERS.\n      a = 3.\n  ENDCASE.\nENDFORM.\n",
        );
        let decisions = facts
            .counts
            .iter()
            .filter_map(|count| count.body.map(|body| body.decisions))
            .collect::<Vec<_>>();
        assert_eq!(decisions, [3]);
    }
}
