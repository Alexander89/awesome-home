#![allow(dead_code)]
use rppal::gpio::Gpio;
use std::time::Duration;
use tokio::time::sleep;

// Gpio uses BCM pin numbering. BCM GPIO 23 is tied to physical pin 16.
const GPIO_PWM: u8 = 12;

// Servo configuration. Change these values based on your servo's verified safe
// minimum and maximum values.
//
// Period: 20 ms (50 Hz). Pulse width: min. 1200 µs, neutral 1500 µs, max. 1800 µs.
const PERIOD_MS: u64 = 20u64;
const PULSE_MIN_US: f32 = 1000.0f32;
const PULSE_NEUTRAL_US: f32 = 1500.00f32;
const PULSE_MAX_US: f32 = 2000.00f32;

fn get_pw(proc: f32) -> u64 {
    let us = (PULSE_MAX_US - PULSE_MIN_US) * proc + PULSE_MIN_US;
    us.floor() as u64
}
fn get_pw_duration(proc: f32) -> Duration {
    Duration::from_micros(get_pw(proc))
}

pub async fn enable_drone() -> Result<(), anyhow::Error> {
    let period_ms = Duration::from_millis(PERIOD_MS);

    println!("switch on drone");
    // Rotate the servo to the opposite side.
    // Retrieve the GPIO pin and configure it as an output.
    let mut pin = Gpio::new()?.get(GPIO_PWM)?.into_output();
    // Enable software-based PWM with the specified period, and rotate the servo by
    // setting the pulse width to its maximum value.
    pin.set_pwm(period_ms.clone(), get_pw_duration(1.00f32))?;

    // Sleep for 500 ms while the servo moves into position.
    // Rotate the servo to the opposite side.
    sleep(Duration::from_millis(1500)).await;
    // Rotate the servo to the opposite side.
    pin.set_pwm(period_ms.clone(), get_pw_duration(0.70f32))?;

    sleep(Duration::from_millis(500)).await;

    pin.set_pwm(period_ms.clone(), get_pw_duration(1.00f32))?;

    sleep(Duration::from_millis(500)).await;
    Ok(())
}
