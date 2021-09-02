use std::path::PathBuf;

use color_eyre::{eyre::eyre, Help, Result};

pub(crate) struct WorkerContext {
    pub(crate) blend_path: PathBuf,
    pub(crate) python_path: PathBuf,
}

impl WorkerContext {
    pub(crate) fn from_dmstl_dir<P>(dmstl_path: P) -> Result<WorkerContext>
    where
        PathBuf: From<P>,
    {
        let dmstl: PathBuf = {
            let mut p = PathBuf::from(dmstl_path);
            if !p.exists() {
                Err(eyre!("Specified displacementMapToStl directory does not exist")
                    .note(format!("Tried \u{201C}{}\u{201D}", p.display()))
                    .note("displacementMapToStl is available on GitHub: https://github.com/Siegler-von-Catan/displacementMapToStl"))
            } else {
                p = p.canonicalize()?;
                Ok(p)
            }
        }?;

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

        Ok(WorkerContext {
            blend_path,
            python_path,
        })
    }
}
