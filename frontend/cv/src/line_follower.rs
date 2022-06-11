use std::fmt::{Display, Formatter};
use anyhow::bail;
use opencv::core::Mat;
use sensor_fusion::state::{MotorState, RobotState};
use crate::OpenCvHandler;
use opencv::*;
use opencv::core::{in_range, Point, Point2d, Rect, Scalar, Size2i, ToInputArray, Vec3b, Vec4i, VecN, Vector};
use opencv::prelude::*;
use opencv::types::{VectorOfPoint, VectorOfPoint2f, VectorOfVec4i, VectorOfVectorOfPoint};
use common::controller::VelocityData;

#[derive(Clone)]
pub struct LineFollower(LineGoal);

impl OpenCvHandler for LineFollower {
    fn handle_frame(&mut self, frame: &Mat) -> anyhow::Result<(VelocityData, String)> {
        line_tracker(frame, self.0).map(|(velo, goal)| {
            self.0 = goal;
            let message = std::format!("{:?}", goal);
            (velo, message)
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LineGoal {
    CenterLine(Option<Direction>),
    FollowLine(Direction),
    LostLine
}

#[derive(Clone, Copy, Debug)]
pub enum Direction {
    Down,
    Left,
    Right
}

fn line_tracker(image: &Mat, goal: LineGoal) -> anyhow::Result<(VelocityData, LineGoal)> {
    let blur = blur(&image)?;
    let (image, mask) = isolate_red(&blur)?;
    let contours = find_contours(&mask)?;

    let contour = contours.iter()
        .max_by(|a, b| f64::total_cmp(
            &contour_area(a).unwrap_or(0.0),
            &contour_area(a).unwrap_or(0.0)
        ));

    // TODO move forwards or backwards depending on amount of lines seen

    if let Some(cnt) = contour {
        if contour_area(&cnt)? > 500.0 {
            let center = find_center(&cnt)?;
            let ratio = point_to_ratio(&center, &image);

            return match goal {
                LineGoal::CenterLine(direction) => {
                    Ok(center_line(ratio.x, ratio.y, direction))
                }
                LineGoal::FollowLine(direction) => {
                    Ok(follow_line(&mask, &cnt, ratio.x, ratio.y, direction))
                }
                LineGoal::LostLine => {
                    Ok(center_line(ratio.x, ratio.y, None))
                }
            }
        } else if let LineGoal::LostLine = goal {
            bail!("No line found");
        } else {
            return Ok((VelocityData::default(), LineGoal::LostLine));
        }
    } else if let LineGoal::LostLine = goal {
        bail!("No line found");
    } else {
        return Ok((VelocityData::default(), LineGoal::LostLine));
    }
}

fn center_line(x: f64, y: f64, next_direction: Option<Direction>) -> (VelocityData, LineGoal) {
    // this has a max of 50% speed
    let error_x = x - 0.5;
    let error_y = y - 0.5;
    let correction_multiplier = 1.0;

    let update = VelocityData {
        forwards_left: 0.0,
        forwards_right: 0.0,
        strafing: (error_x * correction_multiplier) as f32,
        vertical: (-error_y * correction_multiplier) as f32
    };

    let goal = if error_x.abs() < 0.1 && error_y.abs() < 0.1 {
        LineGoal::FollowLine(next_direction.unwrap_or(Direction::Right))
    } else {
        LineGoal::CenterLine(next_direction)
    };

    (update, goal)
}

fn follow_line(mask: &Mat, line: &Contour, x: f64, y: f64, last_direction: Direction) -> (VelocityData, LineGoal) {
    let error_x = x - 0.5;
    let error_y = y - 0.5;
    let correction_multiplier = 1.0;
    let bias_multiplier = 0.3;

    if error_x.abs() > 0.1 && error_y.abs() > 0.1 {
        return (VelocityData::default(), LineGoal::CenterLine(Some(last_direction)));
    }

    let (horizontal_bias, vertical_bias) = match last_direction {
        Direction::Down => (0.0, -1.0),
        Direction::Right => (1.0, 0.0),
        Direction::Left => (-1.0, 0.0),
    };

    let update = VelocityData {
        forwards_left: 0.0,
        forwards_right: 0.0,
        strafing: (error_x * correction_multiplier + horizontal_bias * bias_multiplier) as f32,
        vertical: (-error_y * correction_multiplier + vertical_bias * bias_multiplier) as f32
    };

    let x_start = mask.cols() * 2 / 5;
    let x_end = mask.cols() * 3 / 5;
    let y_start = mask.rows() * 2 / 5;
    let y_end = mask.rows() * 3 / 5;

    let left_trigger = (Point::new(x_start, y_start), Point::new(x_start, y_end));
    let right_trigger = (Point::new(x_end, y_start), Point::new(x_end, y_end));
    let up_trigger = (Point::new(x_start, y_start), Point::new(x_end, y_start));
    let down_trigger = (Point::new(x_start, y_end), Point::new(x_end, y_end));

    let left_triggered = intersect_contour(left_trigger, &line);
    let right_triggered = intersect_contour(right_trigger, &line);
    let up_triggered = intersect_contour(up_trigger, &line);
    let down_triggered = intersect_contour(down_trigger, &line);

    let new_direction = match (last_direction, left_triggered, right_triggered, up_triggered, down_triggered) {
        (Direction::Left, false, true, false, true) => Some(Direction::Down),
        (Direction::Right, true, false, false, true) => Some(Direction::Down),
        (Direction::Down, true, false, true, false) => Some(Direction::Left),
        (Direction::Down, false, true, true, false) => Some(Direction::Right),
        _ => None
    };

    if let Some(new_direction) = new_direction {
        (update, LineGoal::FollowLine(new_direction))
    } else {
        (update, LineGoal::FollowLine(last_direction))
    }
}

fn blur(image: &Mat) -> anyhow::Result<Mat> {
    let mut median = Mat::default();
    imgproc::median_blur(&image, &mut median, 5)?;

    let mut gaussian = Mat::default();
    imgproc::gaussian_blur(&median, &mut gaussian, Size2i::new(5, 5), 0.0, 0.0, core::BORDER_DEFAULT)?;

    Ok(gaussian)
}

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

type Contours = VectorOfVectorOfPoint;
type Contour = VectorOfPoint;

fn find_contours(image: &Mat) -> anyhow::Result<Contours> {
    let mut contours = Contours::default();
    imgproc::find_contours(image, &mut contours, imgproc::RETR_TREE, imgproc::CHAIN_APPROX_SIMPLE, Point::new(0, 0))?;
    Ok(contours)
}

fn find_center(contour: &Contour) -> anyhow::Result<Point2d> {
    let moments = imgproc::moments(contour, false)?;
    let cx = moments.m10 / moments.m00;
    let cy = moments.m01 / moments.m00;

    Ok(Point2d::new(cx, cy))
}

fn point_to_ratio(point: &Point2d, image: &Mat) -> Point2d {
    let x = point.x / image.cols() as f64;
    let y = point.y / image.rows() as f64;
    Point2d::new(x, y)
}

fn truncate_point(point: &Point2d) -> Point {
    Point::new(point.x as _, point.y as _)
}

fn contour_area(contour: &Contour) -> anyhow::Result<f64> {
    Ok(imgproc::contour_area(contour, false)?)
}

// https://stackoverflow.com/questions/3838329/how-can-i-check-if-two-segments-intersect
fn intersect_lines(a: (Point, Point), b: (Point, Point)) -> bool {
    fn ccw(a: Point, b: Point, c: Point) -> bool {
        (c.y-a.y) * (b.x-a.x) > (b.y-a.y) * (c.x-a.x)
    }
    ccw(a.0,b.0,b.1) != ccw(a.1,b.0,b.1) && ccw(a.0,a.1,b.0) != ccw(a.0,a.1,b.1)
}

fn intersect_contour(line: (Point, Point), contour: &Contour) -> bool {
    let mut last_point = None;
    for point in contour.iter() {
        if let Some(last_point) = last_point {
            if intersect_lines(line, (point, last_point)) {
                return true;
            }
        }

        last_point = Some(point);
    }

    false
}
