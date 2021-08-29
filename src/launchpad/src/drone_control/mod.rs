use tello::{CommandMode, Drone};

pub struct DroneControl {
    drone: Option<CommandMode>,
}

impl DroneControl {
    pub fn new() -> Self {
        Self { drone: None }
    }
    pub async fn connect(&mut self, _ssid: String, ip: String) -> Result<(), String> {
        let drone = Drone::new(&*ip).command_mode();
        drone.enable().await?;
        self.drone = Some(drone);
        Ok(())
    }
    pub async fn take_off(&mut self) -> Result<(), String> {
        if let Some(drone) = self.drone.as_ref() {
            drone.take_off().await
        } else {
            Err("no drone connected".to_string())
        }
    }
}
