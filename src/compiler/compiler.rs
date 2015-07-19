use std::path::{Path, PathBuf};
use std::env;

use compiler::SourceFile;
use linker::Linker;

pub struct Compiler<'sf> {
    files: Vec<SourceFile<'sf>>,
    base_path: PathBuf,
    silent: bool,
    verbose: bool
}

impl<'sf> Compiler<'sf> {

    pub fn new(silent: bool, verbose: bool) -> Compiler<'sf> {
        Compiler {
            files: vec![],
            base_path: PathBuf::new(),
            silent: silent,
            verbose: verbose
        }
    }

    pub fn compile_source_files(&mut self, files: &Vec<&str>, verify: bool) -> Result<bool, &str> {

        // Clear any existing source files
        self.files.clear();

        // Set base directory from first source file
        self.base_path = env::current_dir().unwrap();
        self.base_path.push(files.get(0).unwrap());
        self.base_path.set_file_name("");

        println!("Compiling sources: {:?} (Base: {:?})", files, self.base_path);

        // Parse and link files
        self.parse_files(files);
        self.link_files(verify);

        Ok(true)

    }

    pub fn optimize_instructions(&mut self, allow_unsafe: bool) {
        Linker::optimize(allow_unsafe);
    }

    pub fn report_unsused(&self) -> &str {
        "No unused variables or labels."
    }

    pub fn generate_rom_image(&self, file: &str) {
        self.write_out(file, "ROM");
    }

    pub fn generate_symbol_file(&self, file: &str) {
        self.write_out(file, "SYMBOLS");
    }

    pub fn generate_mapping_file(&self, file: &str) {
        self.write_out(file, "MAPPING");
    }

    pub fn generate_json_file(&self, file: &str) {
        self.write_out(file, "JSON DUMP");
    }

    // Private ----------------------------------------------------------------

    fn parse_files(&mut self, files: &Vec<&str>) {
        for s in files {
            self.include_file(s);
        }
    }

    fn include_file(&mut self, path: &str) -> Result<&SourceFile<'sf>, String> {
        match SourceFile::new(None, self.base_path.join(path)) {
            Ok(file) => {
                println!("Including file \"{}\"", path);
                self.files.push(file);
                let source_file = self.files.last_mut().unwrap();
                source_file.parse();
                Ok(source_file)
            },
            Err(err) => Err(err)
        }
    }

    fn link_files(&mut self, verify: bool) {
        Linker::init_files(&mut self.files);
        Linker::link_files(&mut self.files);
    }

    fn write_out(&self, file: &str, content: &str) {
        match file {
            "stdout" => println!("Output {} to standard out", content),
            file => println!("Output {} to file \"{}\"", content, file)
        }
    }

}

