#include <WiFi.h>
#include "esp_eap_client.h"
#include "arduino_secrets.h"   // SECRET_SSID, SECRET_USER, SECRET_PASS

char ssid[] = SECRET_SSID;
char user[] = SECRET_USER;
char pass[] = SECRET_PASS;
char ip[] = SECRET_IP;
uint16_t port = 7878;
WiFiClient client;

struct {
  string room;
  bool multi_frequency;
  int timestemp; //might move to backend
  string frequency;
  string quality;
  int speed;
} Data;

void setup() {
  Serial.begin(115200);
  delay(1000);

  WiFi.mode(WIFI_STA);

  esp_eap_client_set_identity((uint8_t *)user, strlen(user));
  esp_eap_client_set_username((uint8_t *)user, strlen(user));
  esp_eap_client_set_password((uint8_t *)pass, strlen(pass));
  esp_eap_client_set_ttls_phase2_method(ESP_EAP_TTLS_PHASE2_PAP);
  esp_wifi_sta_enterprise_enable();

  Serial.print("Verbinde mit ");
  Serial.println(ssid);
  WiFi.begin(ssid);

  while (WiFi.status() != WL_CONNECTED) {
    delay(500);
    Serial.print(".");
  }

  Serial.println();
  Serial.println("Verbunden");
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

void scan() {

}

void tcp_send() {
  // Verbindung aufbauen, falls noch keine besteht
  if (!client.connected()) {
    Serial.println("Verbinde mit Server...");
    if (!client.connect(ip, port)) {
      Serial.println("Verbindung fehlgeschlagen");
      delay(5000);
      return;
    }
    Serial.println("Mit Server verbunden");
  }

  client.println("S066;false;2000;2,4 GHz;good;300");
  Serial.print("Send");
  
}