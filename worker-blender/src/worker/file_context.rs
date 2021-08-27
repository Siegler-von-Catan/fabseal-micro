use tempfile::{NamedTempFile, TempPath};

use std::{io::Write, path::Path};

use log::{debug, trace};

pub(crate) struct CommandFileContext {
    input_file: TempPath,
    output_file: TempPath,
}

impl CommandFileContext {
    pub(crate) fn create(input_data: &[u8]) -> std::io::Result<CommandFileContext> {
        let mut input_file = NamedTempFile::new()?;
        let output_file = NamedTempFile::new()?;

        trace!("input temp file: {:?}", input_file);
        trace!("output temp file: {:?}", output_file);

        input_file.write_all(input_data)?;

        Ok(CommandFileContext {
            input_file: input_file.into_temp_path(),
            output_file: output_file.into_temp_path(),
        })
    }

    pub(crate) fn input_file_path(&self) -> &Path {
        self.input_file.as_ref()
    }

    pub(crate) fn output_file_path(&self) -> &Path {
        self.output_file.as_ref()
    }

    pub(crate) fn finish(self) -> std::io::Result<Vec<u8>> {
        let result_data = std::fs::read(&self.output_file)?;

        debug!("closing temporary files");
        self.input_file.close()?;
        self.output_file.close()?;

        Ok(result_data)
    }
}
