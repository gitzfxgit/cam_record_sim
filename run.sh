#!/bin/bash

# Wrapper für cargo run mit automatischer Umgebungsvariablen-Konfiguration

export LIBCLANG_PATH=/lib64

cargo run "$@"
