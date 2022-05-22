#include <SoftwareSerial.h>

SoftwareSerial nanoSerial(2, 3); // RX, TX

void setup() {
  Serial.begin(115200);
  nanoSerial.begin(57600);
}

#define LEN 1000

unsigned int read;
byte buf[LEN];

void loop() {
  while (nanoSerial.available()) {
    Serial.write(nanoSerial.read());
  }
  if (read = Serial.available()) {
    read = Serial.readBytes(buf, min(read, LEN));
    // todo parse message
  }
}
