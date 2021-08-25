use tempfile::NamedTempFile;

use std::{io::Write, path::Path};

use anyhow::Result;
use log::{debug, trace};

pub(crate) struct CommandFileContext {
    input_file: NamedTempFile,
    output_file: NamedTempFile,
}

impl CommandFileContext {
    pub(crate) fn create(input_data: &[u8]) -> Result<CommandFileContext> {
        let mut input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;

        trace!("input temp file: {:?}", input_file);
        trace!("output temp file: {:?}", output_file);

        input_file.write_all(input_data)?;

        Ok(CommandFileContext {
            input_file,
            output_file,
        })
    }

    pub(crate) fn input_file_path(&self) -> &Path {
        self.input_file.path()
    }

    pub(crate) fn output_file_path(&self) -> &Path {
        self.output_file.path()
    }

    pub(crate) fn finish(self) -> Result<Vec<u8>> {
        let result_data = std::fs::read(self.output_file.path())?;

        debug!("closing temporary files");
        self.input_file.close()?;
        self.output_file.close()?;

        Ok(result_data)
    }
}
