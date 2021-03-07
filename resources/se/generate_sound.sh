#!/bin/zsh
sox -n -r 8000 plaindrop.wav synth 0.2 sine 200

freq="1000"
multiplier="1.06"
for i in {1..20}; do
	sox -n -r 8000 cleardrop$i.wav synth 0.2 saw "$freq"
	freq="$((freq * multiplier))"
done

freq="2500"
multiplier="1.06"
for i in {1..20}; do
	sox -n -r 8000 attackdrop$i.wav synth 0.2 square "$freq"
	freq="$((freq * multiplier))"
done
