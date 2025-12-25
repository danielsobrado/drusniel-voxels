# Nature Assets Inventory
Generated: 2025-12-25 15:39:26

## Directory Structure

`
assets/models/
|-- rocks/
|   |-- quaternius/
|   |-- kaykit/
|   |-- kenney/
|   -- polypizza/
|
|-- vegetation/
|   |-- trees/
|   |   |-- quaternius/
|   |   |-- kaykit/
|   |   -- kenney/
|   |
|   |-- plants_flowers/
|   |   -- quaternius/
|   |
|   |-- grass_bushes/
|   |   |-- quaternius/
|   |   -- kaykit/
|   |
|   -- crops/
|       -- quaternius/
|
-- props/
    |-- resources/
    |   -- kaykit/
    -- camping/
        -- kenney/
`

## Asset Counts
### Rocks: 0 models
- kaykit: 0
- kenney: 0
- polypizza: 0
- quaternius: 0
### Trees: 0 models
- kaykit: 0
- kenney: 0
- quaternius: 0
### Plants & Flowers: 0 models
- quaternius: 0
### Grass & Bushes: 0 models
- kaykit: 0
- quaternius: 0
### Crops: 0 models
- quaternius: 0
### Props: 0 models
- camping: 0
- resources: 0
## Sources & Licenses

### Quaternius
- Website: https://quaternius.com/
- License: CC0 (Public Domain)
- Formats: FBX, OBJ, glTF
- Style: Stylized/Ghibli-inspired

### Kay Lousberg (KayKit)
- Website: https://kaylousberg.com/game-assets
- License: CC0 (Public Domain)
- Formats: FBX, OBJ, glTF
- Style: Stylized with gradient atlas textures

### Kenney
- Website: https://kenney.nl/
- License: CC0 (Public Domain)
- Formats: OBJ
- Style: Clean, minimalist

### Poly.pizza
- Website: https://poly.pizza/
- License: Varies (check individual models, most are CC0)
- Formats: FBX, OBJ, glTF
- Style: Various

## Usage Notes

All assets from the main sources (Quaternius, KayKit, Kenney) are CC0 licensed:
- Commercial use allowed
- No attribution required
- Modify and redistribute freely

## Integration with Bevy

1. Recommended format: glTF/GLB
2. Fallback: FBX (convert with Blender)
3. Textures: Most use simple atlas textures

## Next Steps

1. Convert FBX/OBJ to glTF if needed
2. Create YAML configs for each asset
3. Set up deterministic placement
4. Apply triplanar shader where needed
