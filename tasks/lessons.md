# Lessons Learned - Image Converter Rust

## Rust - ddsfile crate
- **FourCC Type**: The `ddsfile` crate uses a wrapper type `ddsfile::FourCC(u32)`. To initialize it from bytes, use `u32::from_le_bytes`.
    - Correct: `dds.header.spf.fourcc = Some(ddsfile::FourCC(u32::from_le_bytes(*b"DXT5")));`
    - Incorrect: `dds.header.spf.fourcc = Some(ddsfile::FourCC(*b"DXT5"));`

## Rust - image_dds crate
- **Enum Variants**: For BC3 compression, use `image_dds::ImageFormat::BC3Unorm` (or check the specific crate version). `BC3RgbaUnorm` may not be available in all versions.

## Image Processing
- **DDS Block Alignment**: Always pad images to multiples of 4 pixels when using block compression (BC1-BC7/DXT), otherwise many applications will fail to load the resulting files.
- **Legacy Compatibility**: Many legacy game engines require a Legacy D3D9 Header (FourCC) instead of the modern DX10 header. Manually converting the header is often necessary for maximum compatibility.
## Rust - image crate
- **Trait Imports**: When using methods like `dimensions()` on a `DynamicImage` or `ImageBuffer`, verify if the trait `image::GenericImageView` is strictly required. In newer versions, some methods are available directly on the struct.
- **Mutability**: Some encoders like `TgaEncoder::new` do not require the variable to be marked as `mut` to call `encode()`, as they take a reference to the writer.
