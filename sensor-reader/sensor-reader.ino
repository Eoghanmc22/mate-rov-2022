#include "IMU.h"
#include <avr/wdt.h>

#define DT 16  // Loop time [ms]

byte buff[6];
unsigned long startTime;
int badCount;
bool sendMag = true;
int pressure;

void setup() {
    wdt_disable();
    Serial.begin(9600);  // start serial for output
    badCount = 0;

    initCommunication();
    enableIMU();
    wdt_enable(WDTO_120MS);
}

void loop() {
    wdt_reset();

    startTime = millis();

    // Read pressure data
    pressure = analogRead(A0);
    Serial.print((char) pressure);
    Serial.print((char) (pressure >> 8));

    // Read accelerator data
    readACC(buff);
    handleZero();
    Serial.print((char) buff[0]);
    Serial.print((char) buff[1]);
    Serial.print((char) buff[2]);
    Serial.print((char) buff[3]);
    Serial.print((char) buff[4]);
    Serial.print((char) buff[5]);

    // Read gyro data
    readGYR(buff);
    handleZero();
    Serial.print((char) buff[0]);
    Serial.print((char) buff[1]);
    Serial.print((char) buff[2]);
    Serial.print((char) buff[3]);
    Serial.print((char) buff[4]);
    Serial.print((char) buff[5]);

    if (sendMag) {
         // Read magnetometer data
         readMAG(buff);
         handleZero();
         Serial.print((char) buff[0]);
         Serial.print((char) buff[1]);
         Serial.print((char) buff[2]);
         Serial.print((char) buff[3]);
         Serial.print((char) buff[4]);
         Serial.print((char) buff[5]);
    }
    sendMag = !sendMag;

    //Each loop should be at least DT ms.
    while(millis() - startTime < DT) {
      delay(1);
    }

    // Time to collect data
    Serial.print((char) (millis() - startTime));
    Serial.print((char) 0xFF);
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
