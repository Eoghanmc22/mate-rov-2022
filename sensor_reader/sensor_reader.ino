/*
       This program reads the angles and heading from the accelerometer, gyroscope
       and compass on a BerryIMU connected to an Arduino.

       Feel free to do whatever you like with this code.
       Distributed as-is; no warranty is given.

       https://ozzmaker.com/berryimu/
*/

#include "IMU.h"

#define DT 10     // Loop time [ms]

byte buff[6];
unsigned long startTime;

void setup() {
  Serial.begin(57600); // start serial for output

  detectIMU();
  enableIMU();
}

void loop() {
  startTime = millis();

  readACC(buff);
  Serial.print("A");
  Serial.print((int) buff[1] << 8 | buff[0]);
  Serial.print(",");
  Serial.print((int) buff[3] << 8 | buff[2]);
  Serial.print(",");
  Serial.print((int) buff[5] << 8 | buff[4]);

  readGYR(buff);
  Serial.print(" G");
  Serial.print((int) buff[1] << 8 | buff[0]);
  Serial.print(",");
  Serial.print((int) buff[3] << 8 | buff[2]);
  Serial.print(",");
  Serial.print((int) buff[5] << 8 | buff[4]);

  readMAG(buff);
  Serial.print(" M");
  Serial.print((int) buff[1] << 8 | buff[0]);
  Serial.print(",");
  Serial.print((int) buff[3] << 8 | buff[2]);
  Serial.print(",");
  Serial.print((int) buff[5] << 8 | buff[4]);

  Serial.print(" P");
  Serial.print(analogRead(A0));

  Serial.print(" T");
  Serial.print(millis() - startTime);

  //Each loop should be at least DT ms.
  while(millis() - startTime < DT) {
    delay(1);
  }

  Serial.print(" E");
  Serial.print(millis() - startTime);
  Serial.print('\n');
}
