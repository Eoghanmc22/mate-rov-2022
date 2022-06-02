#include <SoftwareSerial.h>

SoftwareSerial nanoSerial(2, 3); // RX, TX

void setup() {
  Serial.begin(1000000);
  nanoSerial.begin(57600);
}

void loop() {
    while (nanoSerial.available()) {
        Serial.write(nanoSerial.read());
    }
}
