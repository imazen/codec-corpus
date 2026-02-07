import os
import subprocess
from PIL import Image, ImageDraw, ImageFilter
import itertools

# Configuration
OUTPUT_DIR = "tests/corpus"
CWEBP = os.path.abspath("../libwebp/examples/cwebp")
if not os.path.exists(CWEBP):
    print(f"Error: cwebp not found at {CWEBP}")
    exit(1)

def generate_source_images():
    sources = []
    
    # 1. 16x16 Gradient
    img1 = Image.new('RGB', (16, 16))
    for x in range(16):
        for y in range(16):
            img1.putpixel((x, y), (x*16, y*16, (x+y)*8))
    img1.save(f"{OUTPUT_DIR}/src_grad_16.png")
    sources.append(f"{OUTPUT_DIR}/src_grad_16.png")
    
    # 2. 129x131 Checkerboard (Odd dimensions)
    img2 = Image.new('RGB', (129, 131), color='white')
    pixels = img2.load()
    for x in range(129):
        for y in range(131):
            if (x // 16 + y // 16) % 2 == 1:
                pixels[x, y] = (0, 0, 0)
    img2.save(f"{OUTPUT_DIR}/src_checker_odd.png")
    sources.append(f"{OUTPUT_DIR}/src_checker_odd.png")
    
    # 3. 64x64 Noise
    img3 = Image.effect_noise((64, 64), 50).convert('RGB')
    img3.save(f"{OUTPUT_DIR}/src_noise.png")
    sources.append(f"{OUTPUT_DIR}/src_noise.png")

    return sources

def main():
    if not os.path.exists(OUTPUT_DIR):
        os.makedirs(OUTPUT_DIR)
        
    sources = generate_source_images()
    
    # Permutations
    qualities = [0, 50, 90]
    methods = [0, 4] # 0=fast, 4=default (balance)
    filters = [
        [], # default
        ["-f", "0"], # no filter
        ["-f", "50", "-strong"],
        ["-f", "50", "-nostrong"],
    ]
    segments = [
        [],
        ["-segments", "4"], # invoke segment map
        ["-segments", "2", "-sns", "80"],
    ]
    
    count = 0
    for src in sources:
        base_name = os.path.basename(src).replace(".png", "")
        for q, m, f, seg in itertools.product(qualities, methods, filters, segments):
            # Construct output filename
            f_str = "def" if not f else f"f{''.join(f).replace('-','')}"
            s_str = "def" if not seg else f"seg{''.join(seg).replace('-','')}"
            out_name = f"{base_name}_q{q}_m{m}_{f_str}_{s_str}.webp"
            out_path = os.path.join(OUTPUT_DIR, out_name)
            
            cmd = [CWEBP, src, "-o", out_path, "-q", str(q), "-m", str(m)] + f + seg
            
            # Run cwebp (suppress output)
            try:
                subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
                count += 1
                if count % 10 == 0:
                    print(f"Generated {count} images...")
            except subprocess.CalledProcessError as e:
                print(f"Failed to generate {out_name}: {e}")

    print(f"Total corpus size: {count} images.")

if __name__ == "__main__":
    main()
