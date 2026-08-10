use image::RgbImage;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SRCCOPY, SelectObject,
};
use windows::Win32::UI::WindowsAndMessaging::{SM_CXSCREEN, SM_CYSCREEN, GetSystemMetrics};

pub fn size() -> (i32, i32) {
    unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) }
}

pub fn capture() -> Option<RgbImage> {
    let (w, h) = size();
    if w <= 0 || h <= 0 {
        return None;
    }

    unsafe {
        let screen_dc = GetDC(None);
        if screen_dc.is_invalid() {
            return None;
        }
        let mem_dc = CreateCompatibleDC(Some(screen_dc));
        let bitmap = CreateCompatibleBitmap(screen_dc, w, h);
        let old = SelectObject(mem_dc, bitmap.into());

        let copied = BitBlt(mem_dc, 0, 0, w, h, Some(screen_dc), 0, 0, SRCCOPY).is_ok();

        let mut info = BITMAPINFO::default();
        info.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
        info.bmiHeader.biWidth = w;
        info.bmiHeader.biHeight = -h; // 自上而下，避免行序翻转
        info.bmiHeader.biPlanes = 1;
        info.bmiHeader.biBitCount = 32;
        info.bmiHeader.biCompression = BI_RGB.0;

        let mut buf = vec![0u8; (w * h * 4) as usize];
        let got = GetDIBits(
            mem_dc,
            bitmap,
            0,
            h as u32,
            Some(buf.as_mut_ptr() as *mut _),
            &mut info,
            DIB_RGB_COLORS,
        );

        SelectObject(mem_dc, old);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(mem_dc);
        ReleaseDC(Some(HWND::default()), screen_dc);

        if !copied || got == 0 {
            return None;
        }

        // GDI 给的是 BGRA，image 有现成的构造，不必逐像素搬。
        let rgba = image::RgbaImage::from_raw_bgra(w as u32, h as u32, buf)?;
        Some(image::DynamicImage::ImageRgba8(rgba).to_rgb8())
    }
}
