// Outdated!
// See enterprise version for current functionality

#include <WiFi.h>
#include <LiquidCrystal.h>
#include "arduino_secrets.h"   // SECRET_SSID, SECRET_USER, SECRET_PASS

char ssid[] = SECRET_SSID;
char pass[] = SECRET_PASS;
char ip[] = SECRET_IP;
uint16_t port = 7878;
WiFiClient client;
unsigned long lastBeat;

struct Data {
  String ssid;
  String room;
  bool multifrequency;
  String frequency;
  int rssi;
  int channel;
  int speed;
};

Data messen() {
  Data d;
  d.ssid = SECRET_SSID;
  d.room = "S052";
  d.multifrequency = false;
  d.frequency = "2.4"; //Hardcoded till multifrequency is implemented
  d.rssi = WiFi.RSSI();
  d.channel = WiFi.channel();
  d.speed = 300;
  return d;
}

String toLine(const Data &d, char sep = ';') {
  String s;
  s += d.ssid;                                  s += sep;
  s += d.room;                                  s += sep;
  s += (d.multifrequency ? "true" : "false");  s += sep;
  s += d.frequency;                             s += sep;
  s += d.rssi;                                  s += sep;
  s += d.channel;                               s += sep;
  s += d.speed;
  return s;
}

void setup() {
  Serial.begin(115200);
  delay(1000);

  Serial.print("Connecting to: ");
  Serial.println(ssid);
  WiFi.begin(ssid, pass);
  Serial.println(SECRET_SSID);
  Serial.println(SECRET_IP);

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }

  Serial.println();
  Serial.println("Connected");
  printCurrentNet();
  printWifiData();
}

void loop() {
  if (!client.connected()) {
    Serial.println("Connecting to Server...");
    if (!client.connect(ip, port)) {
      Serial.println("Connection failed");
      delay(5000);
      return;
    }
    Serial.println("Connected to Server");
  }

  if (millis() - lastBeat >= 5000) {
  client.println("ping");
  lastBeat = millis();
  Serial.println("Send Ping");
  }

  delay(5000);
  client.println(toLine(messen()));
  lastBeat = millis();
  Serial.println("Send");
}

void printWifiData() {
  Serial.print("IP Address: ");
  Serial.println(WiFi.localIP());

  Serial.print("MAC address: ");
  Serial.println(WiFi.macAddress());
}

void printCurrentNet() {
  Serial.print("SSID: ");
  Serial.println(WiFi.SSID());
}