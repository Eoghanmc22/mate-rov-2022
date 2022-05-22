#include <SoftwareSerial.h>

SoftwareSerial nanoSerial(2, 3); // RX, TX

void setup() {
  Serial.begin(115200);
  nanoSerial.begin(57600);
  Serial.println("hello");
}

#define MSG_START 'V'
#define MSG_END '\n'
#define NUM_SCALE 32768

byte it;
String txBuffer = String();
String rxBuffer = String();


void loop() {
  while (nanoSerial.available()) {
    it = nanoSerial.read();
    //Serial.write(it);

    if (it == '\n' & txBuffer.length() > 0) {
      Serial.print(txBuffer);
      txBuffer.remove(0);
    }
  }
  while (Serial.available()) {
    it = Serial.read();
    rxBuffer += (char)it;
  }

  while (parse_command()) {
    // do something
  }
}

bool parse_command() {
  int start = txBuffer.indexOf(MSG_START);
  int end = txBuffer.indexOf(MSG_START);

  if (start != -1 && end != -1 && start < end) {
    
  }

  return false;
}

class Vector {
  public: 
  float x;
  float y; 
  float z;

  Vector(float x, float y, float z) {
    this->x = x;
    this->y = y;
    this->z = z;
  }

  float length_squared() {
    return this->x * this->x + this->y * this->y + this->z * this->z;
  }

  float length() {
    return sqrt(this->length_squared());
  }

  Vector mul(float a) {
    return Vector(this->x*a, this->y*a, this->z*a);
  }

  Vector div(float a) {
    return Vector(this->x/a, this->y/a, this->z/a);
  }

  float dot(Vector* a) {
    return this->x * a->x + this->y * a->y + this->z * a->z;
  }

  Vector projectOnTo(Vector* a) {
    a->mul(this->dot(a)/a->length_squared());
  }

  Vector normalize() {
    float length = this->length();
    return Vector(this->x/length, this->y/length, this->z/length);
  }
};
