use windows::core::PCWSTR;
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, BITMAPINFO,
    BITMAPINFOHEADER, DIB_RGB_COLORS, RGBQUAD,
};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHGFI_ICON, SHGFI_LARGEICON, SHFILEINFOW};
use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo};
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};
use image::{RgbaImage, ImageFormat};

pub fn extract_icon_as_base64(path: &str) -> Option<String> {
    unsafe {
        let mut shfi = SHFILEINFOW::default();
        let wide_path: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        
        let result = SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            Default::default(),
            Some(&mut shfi),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        if result == 0 || shfi.hIcon.is_invalid() {
            return None;
        }

        let mut icon_info = Default::default();
        if GetIconInfo(shfi.hIcon, &mut icon_info).is_err() {
            let _ = DestroyIcon(shfi.hIcon);
            return None;
        }

        let hdc = GetDC(None);
        let hdc_mem = CreateCompatibleDC(hdc);

        // Extract at 48x48 for better quality and no clipping
        let width = 48;
        let height = 48;

        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // Top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: 0, // BI_RGB
                ..Default::default()
            },
            bmiColors: [RGBQUAD::default(); 1],
        };

        let mut buffer = vec![0u8; (width * height * 4) as usize];
        let lines = GetDIBits(
            hdc_mem,
            icon_info.hbmColor,
            0,
            height as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // Cleanup Windows handles
        let _ = DeleteDC(hdc_mem);
        let _ = ReleaseDC(None, hdc);
        let _ = DeleteObject(icon_info.hbmColor);
        let _ = DeleteObject(icon_info.hbmMask);
        let _ = DestroyIcon(shfi.hIcon);

        if lines == 0 {
            return None;
        }

        // Convert BGRA to RGBA
        for pixel in buffer.chunks_exact_mut(4) {
            pixel.swap(0, 2);
        }

        let img = RgbaImage::from_raw(width as u32, height as u32, buffer)?;
        let mut png_data = Vec::new();
        let mut cursor = Cursor::new(&mut png_data);
        img.write_to(&mut cursor, ImageFormat::Png).ok()?;

        Some(general_purpose::STANDARD.encode(png_data))
    }
}
