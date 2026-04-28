use crate::errors::{GdalError, Result};
use crate::programs::raster::ProgramDestination;
use crate::utils::_last_null_pointer_err;
use crate::Dataset;
use gdal_sys::{GDALWarp, GDALWarpAppOptions};
use std::borrow::Borrow;
use std::ffi::{c_char, c_int, CString};
use std::ptr::{null, null_mut};

/// Wraps a [GDALWarpOptions] object.
///
/// [GDALWarpOptions]: https://gdal.org/en/stable/api/gdal_utils.html#_CPPv420GDALWarpOptions
///
pub struct WarpOptions {
    c_options: *mut GDALWarpAppOptions,
}

impl WarpOptions {
    /// See [GDALWarpAppOptionsNew].
    ///
    /// [GDALWarpAppOptionsNew]: https://gdal.org/en/stable/api/gdal_utils.html#_CPPv421GDALWarpAppOptionsNewPPcP27GDALWarpAppOptionsForBinary
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
                c_options: gdal_sys::GDALWarpAppOptionsNew(c_args.as_mut_ptr(), null_mut()),
            })
        }
    }

    /// Returns the wrapped C pointer
    ///
    /// # Safety
    /// This method returns a raw C pointer
    ///
    pub unsafe fn c_options(&self) -> *mut GDALWarpAppOptions {
        self.c_options
    }
}

impl Drop for WarpOptions {
    fn drop(&mut self) {
        unsafe {
            gdal_sys::GDALWarpAppOptionsFree(self.c_options);
        }
    }
}

impl TryFrom<Vec<&str>> for WarpOptions {
    type Error = GdalError;

    fn try_from(value: Vec<&str>) -> Result<Self> {
        WarpOptions::new(value)
    }
}

pub fn warp<D: Borrow<Dataset>>(
    input: &[D],
    destination: ProgramDestination,
    options: Option<WarpOptions>,
) -> Result<Dataset> {
    _warp(
        &input.iter().map(|x| x.borrow()).collect::<Vec<&Dataset>>(),
        destination,
        options,
    )
}

fn _warp(
    input: &[&Dataset],
    mut destination: ProgramDestination,
    options: Option<WarpOptions>,
) -> Result<Dataset> {
    let (psz_dest_option, h_dst_ds) = match &destination {
        ProgramDestination::Path(c_path) => (Some(c_path), null_mut()),
        ProgramDestination::Dataset { dataset, .. } => (None, dataset.c_dataset()),
    };

    let psz_dest = psz_dest_option.map(|x| x.as_ptr()).unwrap_or_else(null);

    let mut pah_src_ds: Vec<gdal_sys::GDALDatasetH> = input.iter().map(|x| x.c_dataset()).collect();

    let ps_options = options
        .as_ref()
        .map(|x| x.c_options as *const GDALWarpAppOptions)
        .unwrap_or(null());

    let mut pb_usage_error: c_int = 0;

    let dataset_out = unsafe {
        let data = GDALWarp(
            psz_dest,
            h_dst_ds,
            pah_src_ds.len() as c_int,
            pah_src_ds.as_mut_ptr(),
            ps_options,
            &mut pb_usage_error as *mut c_int,
        );

        // GDAL takes the ownership of `h_dst_ds`
        destination.do_no_drop_dataset();

        data
    };

    if dataset_out.is_null() {
        return Err(_last_null_pointer_err("GDALWarp"));
    }

    let result = unsafe { Dataset::from_c_dataset(dataset_out) };

    Ok(result)
}
