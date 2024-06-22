

#[derive(serde::Serialize, serde::Deserialize)]
pub struct PixelGrainDto {
    pub x: i64,
    pub y: i64,
    pub color: String
}