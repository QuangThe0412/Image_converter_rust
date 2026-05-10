use image_dds;
use ddsfile::{Dds, PixelFormatFlags};
use image::RgbaImage;

fn main() {
    let img = RgbaImage::new(64, 64);
    let dds = image_dds::dds_from_image(&img, image_dds::ImageFormat::BC3RgbaUnorm, image_dds::Quality::Fast, image_dds::Mipmaps::None).unwrap();
    
    println!("DDS Header info:");
    println!("FourCC: {:?}", dds.header.spf.fourcc);
    println!("Flags: {:?}", dds.header.spf.flags);
    
    if dds.header10.is_some() {
        println!("DX10 Header is present!");
        let h10 = dds.header10.as_ref().unwrap();
        println!("DXGI Format: {:?}", h10.dxgi_format);
    } else {
        println!("DX10 Header is NOT present.");
    }
}
