use serial::Frame;

fn main() {
    // FIXME GET COM PORT DYNAMICALLY
    let mut data_log = vec![];
    serial::listen_to_port("COM6", move |frame| {
        calibrate_gyro(&frame, &mut data_log);

        Ok(())
    });
}

fn calibrate_gyro(frame: &Frame, data_log: &mut Vec<(f32, f32, f32)>) {
    let g = frame.gyro;
    data_log.push((g.x, g.y, g.z));

    if data_log.len() % 100 == 0 {
        let x: f32 = data_log.iter().map(|(x, y, z)| *x).sum();
        let y: f32 = data_log.iter().map(|(x, y, z)| *y).sum();
        let z: f32 = data_log.iter().map(|(x, y, z)| *z).sum();

        let mut x_list: Vec<f32> = data_log.iter().map(|(x, y, z)| *x).collect();
        let mut y_list: Vec<f32> = data_log.iter().map(|(x, y, z)| *y).collect();
        let mut z_list: Vec<f32> = data_log.iter().map(|(x, y, z)| *z).collect();
        x_list.sort_by(|a, b| a.total_cmp(b));
        y_list.sort_by(|a, b| a.total_cmp(b));
        z_list.sort_by(|a, b| a.total_cmp(b));

        let count = data_log.len() as f32;

        println!("sample mean {}, x: {}, y: {}, z: {}", data_log.len(), x / count, y / count, z / count);
        println!("sample median {}, x: {}, y: {}, z: {}", data_log.len(), x_list[x_list.len() / 2] / count, y_list[y_list.len() / 2] / count, z_list[z_list.len() / 2] / count);
        println!();
    }
}