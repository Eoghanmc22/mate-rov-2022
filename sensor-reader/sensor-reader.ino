#include "IMU.h"
#include <avr/wdt.h>

#define DT 17  // Loop time [ms]

byte buff[6];
unsigned long startTime;
int badCount;
bool sendMag = true;
int pressure;
char check;

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

    check = 0;
    startTime = millis();

    // Read pressure data
    pressure = analogRead(A0);
    writeByte((char) pressure);
    writeByte((char) (pressure >> 8));

    // Read accelerator data
    readACC(buff);
    handleZero();
    writeByte((char) buff[0]);
    writeByte((char) buff[1]);
    writeByte((char) buff[2]);
    writeByte((char) buff[3]);
    writeByte((char) buff[4]);
    writeByte((char) buff[5]);

    // Read gyro data
    readGYR(buff);
    handleZero();
    writeByte((char) buff[0]);
    writeByte((char) buff[1]);
    writeByte((char) buff[2]);
    writeByte((char) buff[3]);
    writeByte((char) buff[4]);
    writeByte((char) buff[5]);

    if (sendMag) {
        // Read magnetometer data
        readMAG(buff);
        handleZero();
        writeByte((char) buff[0]);
        writeByte((char) buff[1]);
        writeByte((char) buff[2]);
        writeByte((char) buff[3]);
        writeByte((char) buff[4]);
        writeByte((char) buff[5]);
    }
    sendMag = !sendMag;

    //Each loop should be at least DT ms.
    //while(millis() - startTime < DT) {
        //delay(1);
    //}

    // Time to collect data
    writeByte((char) (millis() - startTime));
    writeByte((char) check);
    Serial.print((char) 0x6E);
}

void writeByte(char b) {
    if (b == (char) 0x6E) {
        b = (char) 0x6D;
    }

    Serial.print(b);
    check ^= b;
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
