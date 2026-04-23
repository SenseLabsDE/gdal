mod contour;
mod footprint;
mod mdimtranslate;
mod vrt;

pub use contour::*;
pub use footprint::{footprint, FootprintOptions};
pub use mdimtranslate::{multi_dim_translate, MultiDimTranslateOptions};
pub use vrt::*;

use std::ffi::CString;
use std::mem::ManuallyDrop;
use std::path::{Path, PathBuf};

use crate::errors::GdalError;
use crate::utils::_path_to_c_string;
use crate::{errors, Dataset};

pub enum ProgramDestination {
    Path(CString),
    Dataset {
        dataset: ManuallyDrop<Dataset>,
        drop: bool,
    },
}

impl TryFrom<&str> for ProgramDestination {
    type Error = GdalError;

    fn try_from(path: &str) -> errors::Result<Self> {
        Self::path(path)
    }
}

impl TryFrom<&Path> for ProgramDestination {
    type Error = GdalError;

    fn try_from(path: &Path) -> errors::Result<Self> {
        Self::path(path)
    }
}

impl TryFrom<PathBuf> for ProgramDestination {
    type Error = GdalError;

    fn try_from(path: PathBuf) -> errors::Result<Self> {
        Self::path(path)
    }
}

impl From<Dataset> for ProgramDestination {
    fn from(dataset: Dataset) -> Self {
        Self::dataset(dataset)
    }
}

impl Drop for ProgramDestination {
    fn drop(&mut self) {
        match self {
            Self::Path(_) => {}
            Self::Dataset { dataset, drop } => {
                if *drop {
                    unsafe {
                        ManuallyDrop::drop(dataset);
                    }
                }
            }
        }
    }
}

impl ProgramDestination {
    pub fn dataset(dataset: Dataset) -> Self {
        Self::Dataset {
            dataset: ManuallyDrop::new(dataset),
            drop: true,
        }
    }

    pub fn path<P: AsRef<Path>>(path: P) -> errors::Result<Self> {
        let c_path = _path_to_c_string(path.as_ref())?;
        Ok(Self::Path(c_path))
    }

    unsafe fn do_no_drop_dataset(&mut self) {
        match self {
            Self::Path(_) => {}
            Self::Dataset { dataset: _, drop } => {
                *drop = false;
            }
        }
    }
}
