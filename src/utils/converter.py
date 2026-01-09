# hex_text_to_bin.py

input_file = "input.txt"
output_file = "output.bin"

with open(input_file, "r", encoding="utf-8") as f:
    text = f.read()

tokens = text.split()

binary = bytearray()

for t in tokens:
        binary.append(int(t, 16))

with open(output_file, "wb") as f:
    f.write(binary)

print(f"OK: zapisano {len(binary)} bajtów do {output_file}")