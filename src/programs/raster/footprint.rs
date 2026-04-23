use crate::errors::{GdalError, Result};
use crate::programs::raster::ProgramDestination;
use crate::utils::_last_null_pointer_err;
use crate::Dataset;
use gdal_sys::{GDALFootprint, GDALFootprintOptions};
use std::ffi::{c_char, c_int, CString};
use std::ptr::{null, null_mut};

/// Wraps a [GDALFootprintOptions] object.
///
/// [GDALFootprintOptions]: https://gdal.org/en/stable/api/gdal_utils.html#_CPPv420GDALFootprintOptions
///
pub struct FootprintOptions {
    c_options: *mut GDALFootprintOptions,
}

impl FootprintOptions {
    /// See [GDALFootprintOptionsNew].
    ///
    /// [GDALFootprintOptionsNew]: https://gdal.org/en/stable/api/gdal_utils.html#_CPPv423GDALFootprintOptionsNewPPcP29GDALFootprintOptionsForBinary
    ///
    pub fn new<S: Into<Vec<u8>>, I: IntoIterator<Item = S>>(args: I) -> Result<Self> {
        // Convert args to CStrings to add terminating null bytes
        let cstr_args = args
            .into_iter()
            .map(CString::new)
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Self::_new(&cstr_args)
    }

    fn _new(cstr_args: &[CString]) -> Result<Self> {
        // Get pointers to the strings
        let mut c_args = cstr_args
            .iter()
            .map(|x| x.as_ptr() as *mut c_char) // These strings don't actually get modified, the C API is just not const-correct
            .chain(std::iter::once(null_mut())) // Null-terminate the list
            .collect::<Vec<_>>();

        unsafe {
            Ok(Self {
                c_options: gdal_sys::GDALFootprintOptionsNew(c_args.as_mut_ptr(), null_mut()),
            })
        }
    }

    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    ///
    pub unsafe fn c_options(&self) -> *mut GDALFootprintOptions {
        self.c_options
    }
}

impl Drop for FootprintOptions {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::GDALFootprintOptionsFree(self.c_options);
        }
    }
}

impl TryFrom<Vec<&str>> for FootprintOptions {
    type Error = GdalError;

    fn try_from(value: Vec<&str>) -> Result<Self> {
        FootprintOptions::new(value)
    }
}

pub fn footprint(
    input: &Dataset,
    mut destination: ProgramDestination,
    options: Option<FootprintOptions>,
) -> Result<Dataset> {
    let (psz_dest_option, h_dst_ds) = match &destination {
        ProgramDestination::Path(c_path) => (Some(c_path), null_mut()),
        ProgramDestination::Dataset { dataset, .. } => (None, dataset.c_dataset()),
    };

    let psz_dest = psz_dest_option.map(|x| x.as_ptr()).unwrap_or_else(null);

    let src_dataset = input.c_dataset();

    let ps_options = options
        .as_ref()
        .map(|x| x.c_options as *const GDALFootprintOptions)
        .unwrap_or(null());

    let mut pb_usage_error: c_int = 0;

    let dataset_out = unsafe {
        let data = GDALFootprint(
            psz_dest,
            h_dst_ds,
            src_dataset,
            ps_options,
            &mut pb_usage_error as *mut c_int,
        );

        // GDAL takes the ownership of `h_dst_ds`
        destination.do_no_drop_dataset();

        data
    };

    if dataset_out.is_null() {
        return Err(_last_null_pointer_err("GDALFootprint"));
    }

    let result = unsafe { Dataset::from_c_dataset(dataset_out) };

    Ok(result)
}
