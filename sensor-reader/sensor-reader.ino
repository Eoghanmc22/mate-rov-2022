#include "IMU.h"
#include <avr/wdt.h>

#define DT 10  // Loop time [ms]

byte buff[6];
unsigned long startTime;
int badCount;

void setup() {
    wdt_disable();
    Serial.begin(57600);  // start serial for output
    badCount = 0;

    initCommunication();
    enableIMU();
    wdt_enable(WDTO_120MS);
}

void loop() {
    wdt_reset();

    startTime = millis();

    // Read accelerator data
    readACC(buff);
    Serial.print("A");
    Serial.print((int) buff[1] << 8 | buff[0]);
    Serial.print(",");
    Serial.print((int) buff[3] << 8 | buff[2]);
    Serial.print(",");
    Serial.print((int) buff[5] << 8 | buff[4]);
    handleZero();

    // Read gyro data
    readGYR(buff);
    Serial.print(" G");
    Serial.print((int) buff[1] << 8 | buff[0]);
    Serial.print(",");
    Serial.print((int) buff[3] << 8 | buff[2]);
    Serial.print(",");
    Serial.print((int) buff[5] << 8 | buff[4]);
    handleZero();

    // Read magnetometer data
    readMAG(buff);
    Serial.print(" M");
    Serial.print((int) buff[1] << 8 | buff[0]);
    Serial.print(",");
    Serial.print((int) buff[3] << 8 | buff[2]);
    Serial.print(",");
    Serial.print((int) buff[5] << 8 | buff[4]);
    handleZero();

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

void handleZero() {
    for (int i = 0; i < 6; i++) {
        if (buff[i] != 0) {
            badCount = 0;
            return;
        }
    }

    badCount++;

    if (badCount > 5) {
         // illegal function causes reset
         void(* resetFunc) (void) = 0;
         resetFunc();
    }
}
