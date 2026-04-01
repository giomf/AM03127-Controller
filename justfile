default:
    @just --list

# Save the debug build as a raw firmware image.
save-image:
    espflash save-image ./target/riscv32imc-unknown-none-elf/debug/am03127-controller firmware.bin --chip esp32c3

# Upload firmware.bin to the OTA endpoint of the device.
# Usage: just ota <device-ip>
ota ip:
    curl --fail-with-body \
         -X PUT \
         -H "Content-Type: application/octet-stream" \
         --data-binary @firmware.bin \
         http://{{ip}}/ota
