#include "IMU.h"

#define DT 10  // Loop time [ms]

byte buff[6];
unsigned long startTime;

void setup() {
    Serial.begin(57600);  // start serial for output

    // Wait for IMU to initialize
    delay(1000);
    enableIMU();
}

void loop() {
    startTime = millis();

    // Read accelerator data
    readACC(buff);
    Serial.print("A");
    Serial.print((int) buff[1] << 8 | buff[0]);
    Serial.print(",");
    Serial.print((int) buff[3] << 8 | buff[2]);
    Serial.print(",");
    Serial.print((int) buff[5] << 8 | buff[4]);

    // Read gyro data
    readGYR(buff);
    Serial.print(" G");
    Serial.print((int) buff[1] << 8 | buff[0]);
    Serial.print(",");
    Serial.print((int) buff[3] << 8 | buff[2]);
    Serial.print(",");
    Serial.print((int) buff[5] << 8 | buff[4]);

    // Read magnetometer data
    readMAG(buff);
    Serial.print(" M");
    Serial.print((int) buff[1] << 8 | buff[0]);
    Serial.print(",");
    Serial.print((int) buff[3] << 8 | buff[2]);
    Serial.print(",");
    Serial.print((int) buff[5] << 8 | buff[4]);

    // Read pressure data
    Serial.print(" P");
    Serial.print(analogRead(A0));

    // Time to collect data
    Serial.print(" T");
    Serial.print(millis() - startTime);

    //Each loop should be at least DT ms.
    while(millis() - startTime < DT) {
      delay(1);
    }

    // Duration of entire frame
    Serial.print(" E");
    Serial.print(millis() - startTime);
    Serial.print('\n');
}
