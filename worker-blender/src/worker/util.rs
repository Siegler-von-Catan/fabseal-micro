use std::path::PathBuf;

pub(crate) struct Context {
    pub(crate) blend_path: PathBuf,
    pub(crate) python_path: PathBuf,
}
impl Context {
    pub(crate) fn from_dmstl_dir(dmstl_path: &str) -> Context {
        let dmstl: PathBuf = PathBuf::from(dmstl_path).canonicalize().unwrap();

        let blend_path: PathBuf = {
            let mut p = dmstl.clone();
            p.push(r"src/empty.blend");
            p
        };
        let python_path: PathBuf = {
            let mut p = dmstl;
            p.push(r"src/displacementMapToStl.py");
            p
        };

        Context {
            blend_path,
            python_path,
        }
    }
}
