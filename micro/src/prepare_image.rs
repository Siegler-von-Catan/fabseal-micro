use std::cmp::{max, min};

use opencv::{
    core::{Mat, Point2f, Rect, Rect_, Scalar, Scalar_, Size, Vector, CV_8UC1},
    prelude::*,
};

const BORDER_PIXELS: i32 = 2;
const IMG_TYPE: i32 = CV_8UC1;

fn get_mask_on(
    image_size: Size,
    topleft: Point2f,
    downright: Point2f,
) -> opencv::Result<(Mat, Rect)> {
    let center = Point2f {
        x: ((topleft.x + downright.x) / 2.0).round(),
        y: ((topleft.y + downright.y) / 2.0).round(),
    };
    let pts_v = vec![
        center,
        Point2f {
            x: center.x,
            y: topleft.y,
        },
        Point2f {
            x: downright.x,
            y: center.y,
        },
        Point2f {
            x: center.x,
            y: downright.y,
        },
        Point2f {
            x: topleft.x,
            y: center.y,
        },
    ];
    let pts: Vector<Point2f> = pts_v.into();
    let e = opencv::imgproc::fit_ellipse(&pts)?;

    let init: Scalar = Scalar_::all(0.0);
    let red: Scalar = Scalar_::new(255.0, 0.0, 0.0, 0.0);

    let mut mask = Mat::new_size_with_default(image_size, IMG_TYPE, init)?;
    opencv::imgproc::ellipse_rotated_rect(&mut mask, &e, red, -1, 8)?;

    let br = opencv::imgproc::bounding_rect(&pts)?;

    Ok((mask, br))
}

fn process(image: Mat, topleft: Point2f, downright: Point2f) -> opencv::Result<Mat> {
    let sz: Size = image.size()?;

    let (mask, bounding_rect) = get_mask_on(sz, topleft, downright)?;

    let background_color: Scalar = Scalar_::all(255.0);
    let fg = {
        let mut fg = Mat::new_size_with_default(sz, IMG_TYPE, background_color)?;
        image.copy_to_masked(&mut fg, &mask)?;
        fg
    };

    apply_rect_bounds(fg, crop_rect(bounding_rect, sz))
}

fn crop_rect(bounding_rect: Rect, image_size: Size) -> Rect {
    let x = max(0, bounding_rect.x - BORDER_PIXELS);
    let y = max(0, bounding_rect.y - BORDER_PIXELS);
    let w = min(image_size.width, bounding_rect.width + 2 * BORDER_PIXELS);
    let h = min(image_size.height, bounding_rect.height + 2 * BORDER_PIXELS);
    Rect_::new(x, y, w, h)
}

fn apply_rect_bounds(image: Mat, bounds: Rect) -> opencv::Result<Mat> {
    image
        .col_bounds(bounds.x, bounds.x + bounds.width)?
        .row_bounds(bounds.y, bounds.y + bounds.height)
}

fn inner(img: Mat) -> opencv::Result<Mat> {
    let sz = img.size()?;

    let max_width = sz.width - 1 - BORDER_PIXELS;
    let max_height = sz.height - 1 - BORDER_PIXELS;

    let topleft = Point2f::new(BORDER_PIXELS as f32, BORDER_PIXELS as f32);
    let bottomright = Point2f::new(max_width as f32, max_height as f32);

    let rr: Rect_<f32> = Rect_::from_points(topleft, bottomright);

    process(img, topleft, bottomright)
}

pub(crate) fn run(image_buffer: &[u8]) -> opencv::Result<Vec<u8>> {
    let buffer_v: Mat = Mat::from_slice(image_buffer)?;
    let img = opencv::imgcodecs::imdecode(&buffer_v, opencv::imgcodecs::IMREAD_GRAYSCALE)?;

    let result = inner(img)?;

    let mut out_buf = Vector::new();
    opencv::imgcodecs::imencode(".png", &result, &mut out_buf, &Vector::new())?;

    Ok(out_buf.into())
}
