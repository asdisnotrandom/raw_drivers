#!/bin/bash

picotool uf2 convert target/thumbv8m.main-none-eabihf/debug/ida_motor -t elf firmware.uf2
picotool load -x firmware.uf2