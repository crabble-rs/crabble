use std::path::PathBuf;

fn main() {
    let mut directories = vec![PathBuf::from("./asn_tests")];

    while let Some(new_dir) = directories.pop() {
        for entry in std::fs::read_dir(&new_dir).unwrap() {
            let entry = entry.unwrap();
            let entry_type = entry.file_type().unwrap();

            if entry_type.is_dir() {
                directories.push(entry.path());
            }

            if entry_type.is_file() {
                let ext = entry.path();
                let ext = ext
                    .extension()
                    .map(|os_str| os_str.to_string_lossy().to_string())
                    .unwrap_or("".to_string());
                if ext != "asn" {
                    panic!("File in `asn_tests` with invalid extension: {:?}", ext);
                }

                println!("gaming!: {:?}", entry.path());

                let asn = logic::asn::ASN::from_file(entry.path());
                asn.run(true).unwrap();
            }
        }
    }
}
