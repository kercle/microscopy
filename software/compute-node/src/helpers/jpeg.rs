use anyhow::Result;
use image::RgbaImage;
use tokio::task::spawn_blocking;

pub async fn decode_jpeg(data: Vec<u8>) -> Result<RgbaImage> {
    Ok(
        spawn_blocking(move || -> Result<RgbaImage> { Ok(turbojpeg::decompress_image(&data)?) })
            .await??,
    )
}

pub async fn encode_jpeg(image: RgbaImage, quality: i32) -> Result<Vec<u8>> {
    let res = spawn_blocking(move || -> Result<Vec<u8>> {
        let data = turbojpeg::compress_image(&image, quality, turbojpeg::Subsamp::Sub2x2)?;
        Ok(data.to_vec())
    })
    .await??;

    Ok(res)
}
