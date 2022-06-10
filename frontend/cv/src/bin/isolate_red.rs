use opencv::*;
use opencv::core::{in_range, Point, Point2d, Scalar, Size2i, ToInputArray, Vec3b, Vec4i, VecN, Vector};
use opencv::prelude::*;
use opencv::types::{VectorOfPoint2f, VectorOfVec4i, VectorOfVectorOfPoint};
use common::controller::VelocityData;

fn main() -> anyhow::Result<()> {
    let image = imgcodecs::imread("reddd.png", imgcodecs::IMREAD_COLOR)?;

    let mut median = Mat::default();
    imgproc::median_blur(&image, &mut median, 5)?;

    let mut gaussian = Mat::default();
    imgproc::gaussian_blur(&median, &mut gaussian, Size2i::new(5, 5), 0.0, 0.0, core::BORDER_DEFAULT)?;

    let (image, mask) = isolate_red(&gaussian)?;

    let mut contours = VectorOfVectorOfPoint::default();
    imgproc::find_contours(&mask, &mut contours, imgproc::RETR_TREE, imgproc::CHAIN_APPROX_SIMPLE, Point::new(0, 0))?;

    {
        println!("contours: {}", contours.len());

        let mut img2 = image.clone();
        imgproc::draw_contours(&mut img2, &contours, -1, Scalar::from((0.0, 255.0, 0.0)), 1, imgproc::LINE_8, &core::no_array(), i32::MAX, Point::new(0, 0))?;
        highgui::imshow("img2", &img2)?;
    }

    let mut contors = contours.iter().filter(|cnt| imgproc::contour_area(cnt, false).unwrap_or(0.0) >= 500.0);

    if let Some(cnt) = contors.next() {
        let moments = imgproc::moments(&cnt, false)?;
        let cx = moments.m10 / moments.m00;
        let cy = moments.m01 / moments.m00;

        let fx = cx / image.cols() as f64;
        let fy = cy / image.rows() as f64;

        let mut img2 = image.clone();
        imgproc::draw_contours(&mut img2, &contours, -1, Scalar::from((0.0, 255.0, 0.0)), 1, imgproc::LINE_8, &core::no_array(), i32::MAX, Point::new(0, 0))?;
        imgproc::draw_marker(&mut img2, Point::new(cx as i32, cy as i32), Scalar::from((0.0, 255.0, 0.0)), imgproc::MARKER_CROSS, 20, 1, 8)?;
        highgui::imshow("thing", &img2)?;

        // todo set motor speed based off of position
    } else {
        println!("No line found");
    }

    if let Some(_) = contors.next() {
        println!("Multiple canadates found");
    }

    highgui::wait_key(0)?;

    Ok(())
}

fn follower(fx: f64, fy: f64, last: ()) -> VelocityData

fn isolate_red(image: &Mat) -> anyhow::Result<(Mat, Mat)> {
    let mut image_lab = Mat::default();
    imgproc::cvt_color(&image, &mut image_lab, imgproc::COLOR_BGR2Lab, 0)?;

    let mut red = Mat::from_slice::<Vec3b>(&[[0, 0, 230].into()])?;
    let threshold = 50;

    let mut red_lab = Mat::default();
    imgproc::cvt_color(&red, &mut red_lab, imgproc::COLOR_BGR2Lab, 0)?;

    let mut lower_bound = red_lab.at_2d::<Vec3b>(0, 0)?.clone() - Vec3b::all(threshold);
    lower_bound.0[0] = 0;
    let mut upper_bound = red_lab.at_2d::<Vec3b>(0, 0)?.clone() + Vec3b::all(threshold);
    upper_bound.0[0] = 255;

    let mut mask = Mat::default();
    in_range(&image_lab, &lower_bound, &upper_bound, &mut mask)?;

    let mut outa = Mat::default();
    let mut outb = Mat::default();
    imgproc::cvt_color(&image_lab, &mut outa, imgproc::COLOR_Lab2BGR, 0)?;
    core::bitwise_and(&outa, &outa, &mut outb, &mask)?;

    Ok((outb, mask))
}