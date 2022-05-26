void setup() {
  Serial.begin(115200);
  Serial1.begin(57600);
  Serial.println("init");
}

#define MSG_START '`'
#define MSG_END '\n'
#define NUM_SCALE 1000.0

#define VELOCITY_COMMAND 'V'

byte it;
String txBuffer = String();
String rxBuffer = String();

// forwards_left, forwards_right, strafing, up
float velocity_pc[4];

void loop() {
  while (Serial1.available()) {
    it = Serial1.read();
    Serial.write(it);

    if (it == '\n') {
      if (txBuffer.length() > 0 && Serial1.available() < 10) {
        Serial.print(txBuffer);
        txBuffer.remove(0, txBuffer.length());
      }
      Serial.print(MSG_START);
    }
  }
  while (Serial.available()) {
    rxBuffer += (char) Serial.read();
  }

  while (parse_command()) {
    txBuffer += MSG_START;
    txBuffer += "Got command: forwards_left: ";
    txBuffer += velocity_pc[0];
    txBuffer += " forwards_right: ";
    txBuffer += velocity_pc[1];
    txBuffer += " strafing ";
    txBuffer += velocity_pc[2];
    txBuffer += " up: ";
    txBuffer += velocity_pc[3];
    txBuffer += MSG_END;
  }
}

bool parse_command() {
  int start = rxBuffer.indexOf(MSG_START);
  if (start != -1) {
    rxBuffer.remove(0, start);
    int end = rxBuffer.indexOf(MSG_END);

    if (end != -1 && rxBuffer.length() >= 2) {
      char command = rxBuffer[1];
      rxBuffer.remove(0, 2);
      end -= 2;

      switch (command) {
        case VELOCITY_COMMAND: return parse_velocity_command(end);
      }
    }
  }

  return false;
}

bool parse_velocity_command(int end) {
  // + 1 for null termination
  char* working_string = (char*) malloc(end + 1);
  rxBuffer.toCharArray(working_string, end + 1);
  
  char* substring = working_string;
  long check_sum = 0;
  long velocity_new[4];
  int index = 0;
  while (substring != NULL && substring - working_string < end && index < 4) {
    check_sum ^= velocity_new[index] = strtol(substring, &substring, 10);
    index++;
  }
  long expected_checksum = strtol(substring, &substring, 10);

  rxBuffer.remove(0, substring - working_string);

  if (check_sum == expected_checksum) {
    for (int i = 0; i < 4; i++) {
      velocity_pc[i] = (float) velocity_new[i] / NUM_SCALE;
    }
    
    return true;
  }
  return false;
}
