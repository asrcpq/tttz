#!/bin/zsh
t=0.1
freq="200"
sox -n -b 32 -e floating-point -r 8000 plaindrop.wav synth $t sine "$freq" vol -10dB

freq="2000"
sox -n -b 32 -e floating-point -r 8000 cleardrop.wav synth $t square "$freq" vol -10dB

freq="4500"
sox -n -b 32 -e floating-point -r 8000 attackdrop.wav synth $t square "$freq" vol -10dB
