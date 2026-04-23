use crate::cpl::CslStringList;
use crate::errors::Result;
use crate::raster::RasterBand;
use crate::utils::_last_cpl_err;
use crate::vector::{Layer, LayerAccess};
use gdal_sys::{CPLErr, GDALContourGenerateEx};

/// Create vector contours from raster DEM.
///
/// Wraps [GDALContourGenerateEx].
/// See the [program docs] for more details.
///
/// [GDALContourGenerateEx]: https://gdal.org/en/stable/api/gdal_alg.html#_CPPv421GDALContourGenerateEx15GDALRasterBandHPv12CSLConstList16GDALProgressFuncPv
/// [program docs]: https://gdal.org/en/stable/programs/gdal_contour.html
///
// TODO: Add progress func/arg
pub fn contour_generate(
    band: &RasterBand,
    layer: &mut Layer,
    options: CslStringList,
) -> Result<()> {
    let rv = unsafe {
        GDALContourGenerateEx(
            band.c_rasterband(),
            layer.c_layer(),
            options.as_ptr(),
            None,
            std::ptr::null_mut(),
        )
    };

    if rv != CPLErr::CE_None {
        return Err(_last_cpl_err(rv));
    }

    Ok(())
}
