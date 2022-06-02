void writeTo(int device, byte address, byte val);
void readFrom(int device, byte address, int num, byte buff[]);

void initCommunication();
void enableIMU();
void readACC(byte buff[]);
void readMAG(byte buff[]);
void readGYR(byte buff[]);
