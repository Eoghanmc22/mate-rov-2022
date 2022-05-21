#include <Arduino.h>
#include <Wire.h>
#include "LSM6DSL.h"
#include "LIS3MDL.h"

int BerryIMUConnected = 0;

void writeTo(int device, byte address, byte val) {
   Wire.beginTransmission(device); //start transmission to device 
   Wire.write(address);        // send register address
   Wire.write(val);        // send value to write
   Wire.endTransmission(); //end transmission
}

void readFrom(int device, byte address, int num, byte buff[]) {
  int recieved = Wire.requestFrom(device, num, address, 1, true);    // request data from device
  Wire.readBytes(buff, recieved);
}

void detectIMU(){
  //BerryIMUv3 uses the LSM6DSL and LIS3MDL
  Wire.begin(); 
  byte LSM6DSL_WHO_AM_I_response;
  byte LIS3MDL_WHO_AM_I_response;

  byte WHOresponse[2];
  
  //Detect if BerryIMUv3 (Which uses the LSM6DSL and LIS3MDL) is connected
  readFrom(LSM6DSL_ADDRESS, LSM6DSL_WHO_AM_I,1,WHOresponse);
  LSM6DSL_WHO_AM_I_response = WHOresponse[0];
  
  readFrom(LIS3MDL_ADDRESS, LIS3MDL_WHO_AM_I,1,WHOresponse);
  LIS3MDL_WHO_AM_I_response = WHOresponse[0];

  if (LSM6DSL_WHO_AM_I_response == 0x6A && LIS3MDL_WHO_AM_I_response == 0x3D) {
    BerryIMUConnected = 1;
  }
  
  delay(1000);
}


void enableIMU(){
  if(BerryIMUConnected) {
      //initialise the accelerometer
      writeTo(LSM6DSL_ADDRESS,LSM6DSL_CTRL1_XL,0b10011011);        // ODR 3.33 kHz, +/- 4g , BW = 400hz
      writeTo(LSM6DSL_ADDRESS,LSM6DSL_CTRL8_XL,0b11001000);        // Low pass filter enabled, BW9, composite filter
      writeTo(LSM6DSL_ADDRESS,LSM6DSL_CTRL3_C,0b01000100);         // Enable Block Data update, increment during multi byte read

      //initialise the gyroscope
      writeTo(LSM6DSL_ADDRESS,LSM6DSL_CTRL2_G,0b10011100);         // ODR 3.3 kHz, +/- 2000 dps

      //initialise the magnetometer
      writeTo(LIS3MDL_ADDRESS,LIS3MDL_CTRL_REG1, 0b11011100);      // Temp sesnor enabled, High performance, ODR 80 Hz, FAST ODR disabled and Selft test disabled.
      writeTo(LIS3MDL_ADDRESS,LIS3MDL_CTRL_REG2, 0b00100000);      // +/- 8 gauss
      writeTo(LIS3MDL_ADDRESS,LIS3MDL_CTRL_REG3, 0b00000000);      // Continuous-conversion mode

      /*byte temp = 0;
      readFrom(LSM6DSL_ADDRESS,LIS3MDL_CTRL_REG3, 1, &temp);
      Serial.print(temp);*/
  }
}

void readACC(byte buff[]) {
  if (BerryIMUConnected)
    readFrom(LSM6DSL_ADDRESS, LSM6DSL_OUT_X_L_XL, 6, buff);
}
 
void readMAG(byte buff[]) {
  if (BerryIMUConnected) 
    readFrom(LIS3MDL_ADDRESS, 0x80 | LIS3MDL_OUT_X_L, 6, buff);  
}

void readGYR(byte buff[]) {
  if(BerryIMUConnected)  
    readFrom(LSM6DSL_ADDRESS, LSM6DSL_OUT_X_L_G, 6, buff);
}
