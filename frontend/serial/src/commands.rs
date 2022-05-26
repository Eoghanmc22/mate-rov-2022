use glam::*;

pub enum Command {
    /// direction, yaw split
    VelocityUpdate(Vec3, f32)
}

impl Command {
    pub fn to_command_string(&self) -> String {
        let mut string = String::new();
        string.push(common::MSG_START);

        match self {
            Command::VelocityUpdate(velocity, yaw_split) => {
                // fixme make the yaw split math work
                let forwards_left = (velocity.y * (*yaw_split + 1.0) * 1000.0) as u64;
                let forwards_right = (velocity.y * (-*yaw_split + 1.0) * 1000.0) as u64;
                let strafing = (velocity.x * 1000.0) as u64;
                let up = (velocity.z * 1000.0) as u64;
                let check_sum = forwards_left ^ forwards_right ^ strafing ^ up;

                string.push_str(
                    &format!("V{} {} {} {} {}", forwards_left, forwards_right, strafing, up, check_sum)
                );
            }
        }
        string.push(common::MSG_END);

        string
    }
}
