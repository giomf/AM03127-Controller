default:
    @just --list

# Build the release firmware and save it as a raw image (firmware.bin) for OTA.
# Set WIFI_SSID and WIFI_PASS first, they are compiled in.
save-image:
    cd am03127-firmware && cargo build --release
    espflash save-image ./am03127-firmware/target/riscv32imc-unknown-none-elf/release/am03127-firmware firmware.bin --chip esp32c3 --partition-table ./am03127-firmware/partition-table.csv

# Upload firmware.bin to the OTA endpoint of the device.
# Usage: just ota <device-ip>
ota ip:
    curl --fail-with-body \
         -X PUT \
         -H "Content-Type: application/octet-stream" \
         --data-binary @firmware.bin \
         http://{{ip}}/ota

# Serve the web UI with live reload against the mock panel (am03127-webui/mock.py).
webui:
    #!/usr/bin/env bash
    python3 am03127-webui/mock.py & trap "kill $!" EXIT
    cd am03127-webui && trunk serve --port 8080
