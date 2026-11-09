#include <WiFi.h>
#include "esp_eap_client.h"
#include "arduino_secrets.h"   // SECRET_SSID, SECRET_USER, SECRET_PASS

char ssid[] = SECRET_SSID;
char user[] = SECRET_USER;
char pass[] = SECRET_PASS;
char ip[] = SECRET_IP;
uint16_t port = 7878;
WiFiClient client;

struct Data {
  String room;
  bool multifrequency;
  String frequency;
  int rssi;
  int channel;
  int speed;
};

Data messen() {
  Data d;
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

  WiFi.mode(WIFI_STA);

  esp_eap_client_set_identity((uint8_t *)user, strlen(user));
  esp_eap_client_set_username((uint8_t *)user, strlen(user));
  esp_eap_client_set_password((uint8_t *)pass, strlen(pass));
  esp_eap_client_set_ttls_phase2_method(ESP_EAP_TTLS_PHASE2_PAP);
  esp_wifi_sta_enterprise_enable();

  Serial.print("Connecting to: ");
  Serial.println(ssid);
  WiFi.begin(ssid);

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
  tcp_send();
  delay(5000);
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

void tcp_send() {
  if (!client.connected()) {
    Serial.println("Connecting to Server...");
    if (!client.connect(ip, port)) {
      Serial.println("Connection failed");
      delay(5000);
      return;
    }
    Serial.println("Connected to Server");
  }

  Data d = messen();
  client.println(toLine(d));
  Serial.println("Send");
}