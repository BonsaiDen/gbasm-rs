use compiler::SourceFile;

pub struct Linker;

impl Linker {

    pub fn optimize(allow_unsafe: bool) {
        println!("Optimizing instructions (unsafe={})", allow_unsafe);
    }

    pub fn init_files(files: &mut Vec<SourceFile>) {
        for f in files.iter_mut() {
            f.id = 0;
        }
    }

    pub fn link_files(files: &mut Vec<SourceFile>) {
        for f in files.iter_mut() {
            f.id = 1;
        }
    }

}

