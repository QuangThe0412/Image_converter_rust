# Implementation Plan - Fix TGA and DDS Compatibility

## 1. Research & Investigation
- [x] Research BOI/Cube Engine TGA requirements. <!-- id: 0 -->
- [x] Confirm POT (Power of Two) dimension requirements for game textures. <!-- id: 1 -->

## 2. Implementation
- [x] Implement `pad_to_power_of_two` to ensure compatible dimensions. <!-- id: 2 -->
- [x] Refactor `save_tga_uncompressed` to use `TgaEncoder` with RLE disabled. <!-- id: 3 -->
- [x] Apply POT padding to TGA, DDS, and DDJ formats. <!-- id: 4 -->

## 3. Verification
- [x] Verified code builds and handles padding logic. <!-- id: 5 -->
- [x] Fixed potential orientation and compression issues in TGA. <!-- id: 6 -->
