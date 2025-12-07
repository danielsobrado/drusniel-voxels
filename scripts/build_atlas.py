"""
Texture Atlas Builder for Voxel Builder

This script combines individual texture files into a single atlas.png file.
The atlas is a 4x4 grid (16 tiles) at 256x256 pixels per tile = 1024x1024 total.

Usage:
    python build_atlas.py

Requirements:
    pip install Pillow
"""

from PIL import Image
import os
from pathlib import Path

# Configuration
TILE_SIZE = 256
ATLAS_COLUMNS = 4
ATLAS_ROWS = 4
ATLAS_SIZE = TILE_SIZE * ATLAS_COLUMNS

# Texture directory (relative to script)
SCRIPT_DIR = Path(__file__).parent
TEXTURES_DIR = SCRIPT_DIR.parent / "assets" / "textures"
OUTPUT_FILE = TEXTURES_DIR / "atlas.png"

# Texture mapping: (atlas_index, search_pattern or exact filename)
# Atlas layout:
# Row 0: [0] Grass Top, [1] Dirt, [2] Stone, [3] Bedrock
# Row 1: [4] Sand, [5] Clay, [6] Water, [7] Grass Side
# Row 2: [8] Wood/Bark, [9] Leaves, [10] reserved, [11] reserved
# Row 3: [12] reserved, [13] reserved, [14] reserved, [15] reserved

TEXTURE_MAPPINGS = [
    # Index 0: Grass Top
    (0, ["Fantasy Grass Texture", "grass_top"]),
    # Index 1: Dirt/Earth
    (1, ["Fantasy Earth Texture", "dirt", "earth"]),
    # Index 2: Stone/Cobblestone
    (2, ["Cobblestone Texture", "stone", "rock"]),
    # Index 3: Bedrock
    (3, ["Bedrock Texture", "bedrock"]),
    # Index 4: Sand
    (4, ["Sand Texture", "sand"]),
    # Index 5: Clay
    (5, ["Terracotta Clay Texture", "clay"]),
    # Index 6: Water
    (6, ["Water Texture", "water"]),
    # Index 7: Grass Side (dirt with grass edge)
    (7, ["Minecraft Grass Block", "grass_side"]),
    # Index 8: Wood/Bark
    (8, ["Oak Bark Texture", "bark", "wood"]),
    # Index 9: Leaves
    (9, ["Oak Leaf Texture", "leaf", "leaves"]),
]


def find_texture_file(patterns: list[str]) -> Path | None:
    """Find a texture file matching any of the given patterns."""
    for file in TEXTURES_DIR.iterdir():
        if file.suffix.lower() in ['.png', '.jpg', '.jpeg']:
            filename_lower = file.stem.lower()
            for pattern in patterns:
                if pattern.lower() in filename_lower:
                    return file
    return None


def create_placeholder(color: tuple[int, int, int]) -> Image.Image:
    """Create a solid color placeholder tile."""
    img = Image.new('RGBA', (TILE_SIZE, TILE_SIZE), color + (255,))
    return img


def resize_texture(img: Image.Image) -> Image.Image:
    """Resize texture to tile size, maintaining aspect ratio and cropping if needed."""
    # Convert to RGBA
    if img.mode != 'RGBA':
        img = img.convert('RGBA')
    
    # Resize to fit, then crop to exact size
    aspect = img.width / img.height
    
    if aspect > 1:
        # Wider than tall
        new_height = TILE_SIZE
        new_width = int(TILE_SIZE * aspect)
    else:
        # Taller than wide
        new_width = TILE_SIZE
        new_height = int(TILE_SIZE / aspect)
    
    img = img.resize((new_width, new_height), Image.Resampling.LANCZOS)
    
    # Center crop
    left = (new_width - TILE_SIZE) // 2
    top = (new_height - TILE_SIZE) // 2
    img = img.crop((left, top, left + TILE_SIZE, top + TILE_SIZE))
    
    return img


def build_atlas():
    """Build the texture atlas from individual texture files."""
    print(f"Building texture atlas...")
    print(f"Textures directory: {TEXTURES_DIR}")
    print(f"Output: {OUTPUT_FILE}")
    print(f"Atlas size: {ATLAS_SIZE}x{ATLAS_SIZE} ({ATLAS_COLUMNS}x{ATLAS_ROWS} tiles at {TILE_SIZE}px each)")
    print()
    
    # Create blank atlas
    atlas = Image.new('RGBA', (ATLAS_SIZE, ATLAS_SIZE), (0, 0, 0, 255))
    
    # Placeholder colors for missing textures
    placeholder_colors = [
        (100, 180, 100),  # 0: Grass - green
        (139, 90, 43),    # 1: Dirt - brown
        (128, 128, 128),  # 2: Stone - gray
        (40, 40, 40),     # 3: Bedrock - dark gray
        (238, 214, 175),  # 4: Sand - tan
        (180, 100, 60),   # 5: Clay - orange-brown
        (64, 164, 223),   # 6: Water - blue
        (100, 140, 80),   # 7: Grass side - greenish brown
        (101, 67, 33),    # 8: Wood - dark brown
        (34, 139, 34),    # 9: Leaves - forest green
        (64, 64, 64),     # 10: Reserved - opaque gray (no accidental transparency)
        (64, 64, 64),     # 11: Reserved - opaque gray
        (64, 64, 64),     # 12: Reserved - opaque gray
        (64, 64, 64),     # 13: Reserved - opaque gray
        (64, 64, 64),     # 14: Reserved - opaque gray
        (64, 64, 64),     # 15: Reserved - opaque gray
    ]
    
    # Process each texture
    for index, patterns in TEXTURE_MAPPINGS:
        col = index % ATLAS_COLUMNS
        row = index // ATLAS_COLUMNS
        x = col * TILE_SIZE
        y = row * TILE_SIZE
        
        texture_file = find_texture_file(patterns)
        
        if texture_file:
            print(f"[{index}] Found: {texture_file.name}")
            try:
                img = Image.open(texture_file)
                img = resize_texture(img)
                atlas.paste(img, (x, y))
            except Exception as e:
                print(f"    Error loading: {e}, using placeholder")
                placeholder = create_placeholder(placeholder_colors[index] if index < len(placeholder_colors) else (128, 128, 128))
                atlas.paste(placeholder, (x, y))
        else:
            print(f"[{index}] Not found: {patterns[0]}, using placeholder")
            placeholder = create_placeholder(placeholder_colors[index] if index < len(placeholder_colors) else (128, 128, 128))
            atlas.paste(placeholder, (x, y))
    
    # Fill remaining slots with opaque placeholders (avoid transparent tiles bleeding into the atlas)
    for index in range(len(TEXTURE_MAPPINGS), ATLAS_COLUMNS * ATLAS_ROWS):
        col = index % ATLAS_COLUMNS
        row = index // ATLAS_COLUMNS
        x = col * TILE_SIZE
        y = row * TILE_SIZE
        placeholder = create_placeholder(placeholder_colors[index] if index < len(placeholder_colors) else (128, 128, 128))
        atlas.paste(placeholder, (x, y))
    
    # Save atlas
    atlas.save(OUTPUT_FILE, 'PNG')
    print()
    print(f"Atlas saved to: {OUTPUT_FILE}")
    print(f"  Size: {atlas.width}x{atlas.height}")


if __name__ == "__main__":
    build_atlas()
