#include <SoftwareSerial.h>

SoftwareSerial nanoSerial(2, 3); // RX, TX

void setup() {
  Serial.begin(57600);
  nanoSerial.begin(57600);
}

void loop() {
  if (nanoSerial.available())
    Serial.write(nanoSerial.read());
  if (Serial.available())
    nanoSerial.write(Serial.read());
}
