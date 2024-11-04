use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    // Ścieżka do wygenerowanego pliku
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("generated_tests.rs");

    let mut file = fs::File::create(&dest_path).expect("Could not create generated_tests.rs");

    // Początek generowanego kodu: importowanie traitów
    writeln!(file, "// Auto generated code.\n").unwrap();

    // Iterujemy przez pliki w `src/common`, które zaczynają się od `test`
    for entry in fs::read_dir("src/common").expect("Could not read common directory") {
        let entry = entry.expect("Could not read directory entry");
        let path = entry.path();
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("Invalid file name");

        // Sprawdzamy, czy nazwa pliku zaczyna się od `test`
        if file_name.starts_with("test") && path.extension().and_then(|s| s.to_str()) == Some("rs")
        {
            // Importuj trait `TestCaseTrait` z tego modułu, używając pełnej ścieżki
            writeln!(file, "use crate::common::{}::TestCaseTrait;", file_name).unwrap();
        }
    }

    // Implementacja `TestSuiteCommon` dla `TestSuite`
    writeln!(
        file,
        "impl crate::common::TestSuiteCommon for crate::common::TestSuite {{"
    )
    .unwrap();
    writeln!(file, "    fn run(&self) {{").unwrap();

    // Iterujemy przez pliki ponownie, aby stworzyć ciało funkcji `run`
    for entry in fs::read_dir("src/common").expect("Could not read common directory") {
        let entry = entry.expect("Could not read directory entry");
        let path = entry.path();
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("Invalid file name");

        // Sprawdzamy, czy nazwa pliku zaczyna się od `test`
        if file_name.starts_with("test") && path.extension().and_then(|s| s.to_str()) == Some("rs")
        {
            // Dodaj kod do tworzenia instancji i wywoływania `run`
            writeln!(
                file,
                "        let test_case = crate::common::{}::TestCase {{ tmp: String::from(\"value\") }};",
                file_name
            )
            .unwrap();
            writeln!(file, "        test_case.run();").unwrap();
        }
    }

    // Zamknięcie funkcji `run`
    writeln!(file, "    }}").unwrap();
    writeln!(file, "}}").unwrap();

    println!("cargo:rerun-if-changed=src/common");
}
