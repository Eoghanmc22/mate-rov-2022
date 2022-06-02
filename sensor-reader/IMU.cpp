#include <Arduino.h>
#include <Wire.h>
#include "LSM6DSL.h"
#include "LIS3MDL.h"

void writeTo(int device, byte address, byte val) {
    Wire.beginTransmission(device);  // Start transmission to device

    Wire.write(address);             // Send register address
    Wire.write(val);                 // Send value to write

    Wire.endTransmission();          // End transmission
}

void readFrom(int device, byte address, int num, byte buff[]) {
    int received = Wire.requestFrom(device, num, address, 1, true);  // Request data from device
    Wire.readBytes(buff, received);
}

void initCommunication(){
  Wire.begin();
  
  // Wait for IMU to power on
  delay(1000);
}

void enableIMU(){
    // Initialise the accelerometer
    writeTo(LSM6DSL_ADDRESS,LSM6DSL_CTRL1_XL,0b10011011);    // ODR 3.33 kHz, +/- 4g, BW = 400hz
    writeTo(LSM6DSL_ADDRESS,LSM6DSL_CTRL8_XL,0b11001000);    // Low pass filter enabled, BW9, composite filter
    writeTo(LSM6DSL_ADDRESS,LSM6DSL_CTRL3_C,0b01000100);     // Enable Block Data update, increment during multi byte read

    // Initialise the gyroscope
    writeTo(LSM6DSL_ADDRESS,LSM6DSL_CTRL2_G,0b10011100);     // ODR 3.3 kHz, +/- 2000 dps

    // Initialise the magnetometer
    writeTo(LIS3MDL_ADDRESS,LIS3MDL_CTRL_REG1, 0b11011100);  // Temp sensor enabled, High performance, ODR 80 Hz, FAST ODR disabled and Self test disabled.
    writeTo(LIS3MDL_ADDRESS,LIS3MDL_CTRL_REG2, 0b00100000);  // +/- 8 gauss
    writeTo(LIS3MDL_ADDRESS,LIS3MDL_CTRL_REG3, 0b00000000);  // Continuous-conversion mode
}

void readACC(byte buff[]) {
    readFrom(LSM6DSL_ADDRESS, LSM6DSL_OUT_X_L_XL, 6, buff);
}
 
void readMAG(byte buff[]) {
    readFrom(LIS3MDL_ADDRESS, 0x80 | LIS3MDL_OUT_X_L, 6, buff);  
}

void readGYR(byte buff[]) {
    readFrom(LSM6DSL_ADDRESS, LSM6DSL_OUT_X_L_G, 6, buff);
}
