# Asset Download Guide

This guide explains how to download all the textures and models for the voxel game project.

## ✅ Automated Downloads (Complete)

The following terrain textures from **ambientCG** have been automatically downloaded and are ready to use:

- ✅ Grass (Grass001_1K-JPG)
- ✅ Dirt (Ground037_1K-JPG)
- ✅ Rock (Rock030_1K-JPG)
- ✅ Sand (Ground054_1K-JPG)
- ✅ Tilled Soil (Ground048_1K-JPG)

## 📥 Manual Downloads Required

### 3DTextures.me Assets (Google Drive)

The script has identified all the Google Drive links for 3DTextures.me assets. Follow these steps:

#### Building Materials

1. **Wood Planks** (Stylized Wood Wall 001)
   - 🔗 [Download from Google Drive](https://drive.google.com/drive/folders/1mKBGerzy1BKQvjCkyMV2BmfAoIdXnWQh)
   - Save as: `temp/guide-downloads/gdrive/Stylized_Wood_Wall_001.zip`
   - Destination: `assets/pbr/buildings/wood_plank/`

2. **Stone Brick** (Stylized Stone Wall 001)
   - 🔗 [Download from Google Drive](https://drive.google.com/drive/folders/1QGWGddAUA1m7bBxpp-mM32hYUbABbv_H)
   - Save as: `temp/guide-downloads/gdrive/Stylized_Stone_Wall_001.zip`
   - Destination: `assets/pbr/buildings/stone_brick/`

3. **Metal Plates** (Stylized Metal Plates 001)
   - 🔗 [Download from Google Drive](https://drive.google.com/drive/folders/1zAczmGVyzWK7PuF9D93yUzWlchCHjCrZ)
   - Save as: `temp/guide-downloads/gdrive/Stylized_Metal_Plates_001.zip`
   - Destination: `assets/pbr/buildings/metal_plate/`

4. **Thatch/Straw Roof** (Thatched Roof 001)
   - 🔗 [Download from Google Drive](https://drive.google.com/drive/folders/1L20U-CXcEG-2zK2a5Ybe0SP7HR1gnk9o)
   - Save as: `temp/guide-downloads/gdrive/Thatched_Roof_001.zip`
   - Destination: `assets/pbr/buildings/thatch/`

5. **Wood Shingles** (Stylized Wood Shingles 001)
   - 🔗 [Download from Google Drive](https://drive.google.com/drive/folders/1TsVtvlSzuQVTIOm6vDpzKGaSmQUko21o)
   - Save as: `temp/guide-downloads/gdrive/Stylized_Wood_Shingles_001.zip`
   - Destination: `assets/pbr/buildings/wood_shingles/`

#### Props & Environment

6. **Large Rocks/Cliff** (Stylized Cliff Rock 001)
   - 🔗 [Download from Google Drive](https://drive.google.com/drive/folders/1wwM0_OpPZlebhKD1mRwGjMXu6fNk1BHL)
   - Save as: `temp/guide-downloads/gdrive/Stylized_Cliff_Rock_001.zip`
   - Destination: `assets/pbr/props/rocks/rock_large/`

7. **Cobblestone** (Cobblestone Irregular Floor 001)
   - 🔗 [Download from Google Drive](https://drive.google.com/drive/folders/1pgRNS3ZAgTZyDb0w6BzWyW9ZNYha6FTH)
   - Save as: `temp/guide-downloads/gdrive/Cobblestone_Irregular_Floor_001.zip`
   - Destination: `assets/pbr/props/cobblestone/`

8. **Wooden Crate** (Stylized Crate 001)
   - 🔗 [Download from Google Drive](https://drive.google.com/drive/folders/1aL5FywCnBOxLbUWI0JWubh23WwQ4lBkr)
   - Save as: `temp/guide-downloads/gdrive/Stylized_Crate_001.zip`
   - Destination: `assets/pbr/props/containers/crate/`

### Water Texture (ambientCG)

9. **Water** - The Water002 asset ID doesn't exist on ambientCG
   - 🔗 [Search for water textures](https://ambientcg.com/)
   - Search for "water" and find a suitable water normal map
   - Download and save to: `assets/pbr/water/flow_normal.png`

### Crops Models (Quaternius)

10. **Ultimate Crops Pack**
    - 🔗 [Poly.pizza Download](https://poly.pizza/m/Ro6K0Yg7mx)
    - 🔗 [Quaternius.com Download](https://quaternius.com/packs/ultimatecrops.html)
    - Contains 100+ crop models in 5 growth stages
    - Formats: FBX, OBJ, Blend
    - Destination: `assets/models/crops/`

## 🚀 Quick Start Instructions

### Step 1: Run the Download Script

```powershell
.\scripts\download-guide-assets.ps1
```

This will:
- Automatically download all ambientCG terrain textures ✅
- Display Google Drive links for 3DTextures.me assets
- Show instructions for manual downloads

### Step 2: Download Google Drive Assets

For each Google Drive link above:

1. Click the link to open the Google Drive folder
2. Click the "Download" button (folder icon with down arrow)
3. Save the ZIP file to the exact path shown (e.g., `temp/guide-downloads/gdrive/Stylized_Wood_Wall_001.zip`)
4. The folder will be downloaded as a ZIP automatically

### Step 3: Re-run the Script

After downloading the ZIP files:

```powershell
.\scripts\download-guide-assets.ps1
```

The script will automatically:
- Detect the downloaded ZIP files
- Extract them
- Copy the texture maps to the correct asset folders
- Rename files as needed (e.g., `*_basecolor.png` → `albedo.png`)
- Convert JPG files to PNG if necessary

## 📁 Expected Folder Structure

After all downloads are complete, you should have:

```
assets/
├── pbr/
│   ├── buildings/
│   │   ├── metal_plate/
│   │   ├── stone_brick/
│   │   ├── thatch/
│   │   ├── wood_plank/
│   │   └── wood_shingles/
│   ├── props/
│   │   ├── cobblestone/
│   │   ├── containers/crate/
│   │   └── rocks/rock_large/
│   ├── terrain/
│   │   ├── dirt/
│   │   ├── grass/
│   │   ├── rock/
│   │   ├── sand/
│   │   └── tilled_soil/
│   └── water/
└── models/
    └── crops/
        ├── wheat/
        ├── carrot/
        └── corn/
```

Each texture folder should contain:
- `albedo.png` - Base color/diffuse map
- `normal.png` - Normal map for surface detail
- `roughness.png` - Roughness/smoothness map
- `ao.png` - Ambient occlusion map
- `metallic.png` - Metallic map (for metal materials only)

## ℹ️ Notes

- All textures from 3DTextures.me are **free** (CC0 license)
- 1024x1024 resolution is included for free
- 4K versions available for Patreon supporters
- All textures from ambientCG are **free** (CC0 license)
- Ultimate Crops Pack from Quaternius is **free** (CC0 license)

## 🐛 Troubleshooting

**Problem:** Script says "ZIP not found"
- **Solution:** Make sure you saved the ZIP to the exact path shown by the script

**Problem:** Textures extracted but maps missing
- **Solution:** Check the extracted folder structure - the script looks for files matching patterns like `*_basecolor.*`, `*_normal.*`, etc.

**Problem:** Google Drive download button not working
- **Solution:** Make sure you're logged into a Google account, or try a different browser

**Problem:** Water texture failed to download
- **Solution:** The Water002 asset ID doesn't exist. Visit ambientCG.com and search for any water texture with normal maps
