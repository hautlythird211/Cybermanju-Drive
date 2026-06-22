use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSheetResult {
    pub data: Vec<u8>,
    pub grid: String,
    pub tile_size: u32,
    pub thumb_count: u32,
    pub width: u32,
    pub height: u32,
}

/// Builds sprite sheets by packing multiple thumbnail tiles into a single image.
pub struct SpriteSheetBuilder {
    tile_size: u32,
    grid_cols: u32,
    max_tiles: u32,
    tiles: Vec<Vec<u8>>,
}

impl SpriteSheetBuilder {
    pub fn new(tile_size: u32, grid_cols: u32) -> Self {
        Self {
            tile_size,
            grid_cols,
            max_tiles: grid_cols * grid_cols,
            tiles: Vec::new(),
        }
    }

    /// Add a pre-decoded RGBA tile.
    pub fn add_tile(&mut self, rgba: &[u8], w: u32, h: u32) -> Result<usize> {
        if self.tiles.len() >= self.max_tiles as usize {
            return Err(anyhow::anyhow!(
                "sprite sheet full: {} tiles max",
                self.max_tiles
            ));
        }
        let mut tile = rgba.to_vec();
        // Resize to tile_size x tile_size if needed (simple nearest-neighbor)
        if w != self.tile_size || h != self.tile_size {
            tile = resize_rgba(rgba, w, h, self.tile_size, self.tile_size);
        }
        let idx = self.tiles.len();
        self.tiles.push(tile);
        Ok(idx)
    }

    /// Pack all tiles into a single RGBA canvas, then encode as PNG.
    pub fn build(&self) -> Result<SpriteSheetResult> {
        let tile_count = self.tiles.len() as u32;
        let rows = (tile_count + self.grid_cols - 1) / self.grid_cols;
        let width = self.grid_cols * self.tile_size;
        let height = rows * self.tile_size;

        let mut canvas = vec![0u8; (width * height * 4) as usize];

        for (i, tile) in self.tiles.iter().enumerate() {
            let col = (i as u32) % self.grid_cols;
            let row = (i as u32) / self.grid_cols;
            let x_offset = col * self.tile_size;
            let y_offset = row * self.tile_size;

            for ty in 0..self.tile_size {
                for tx in 0..self.tile_size {
                    let src_idx = ((ty * self.tile_size + tx) * 4) as usize;
                    let dst_idx = ((y_offset + ty) * width + x_offset + tx) * 4;
                    if src_idx + 3 < tile.len() && ((dst_idx + 3) as usize) < canvas.len() {
                        canvas[dst_idx as usize] = tile[src_idx];
                        canvas[(dst_idx + 1) as usize] = tile[src_idx + 1];
                        canvas[(dst_idx + 2) as usize] = tile[src_idx + 2];
                        canvas[(dst_idx + 3) as usize] = tile[src_idx + 3];
                    }
                }
            }
        }

        // Encode as PNG
        let img = image::RgbaImage::from_raw(width, height, canvas)
            .ok_or_else(|| anyhow::anyhow!("failed to create sprite sheet image"))?;
        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png)?;
        let data = buf.into_inner();

        Ok(SpriteSheetResult {
            data,
            grid: format!("{}x{}", self.grid_cols, rows),
            tile_size: self.tile_size,
            thumb_count: tile_count,
            width,
            height,
        })
    }
}

/// Simple nearest-neighbor RGBA resize.
fn resize_rgba(input: &[u8], src_w: u32, src_h: u32, dst_w: u32, dst_h: u32) -> Vec<u8> {
    let mut output = vec![0u8; (dst_w * dst_h * 4) as usize];
    for y in 0..dst_h {
        for x in 0..dst_w {
            let src_x = (x * src_w / dst_w) as usize;
            let src_y = (y * src_h / dst_h) as usize;
            let src_idx = (src_y * src_w as usize + src_x) * 4;
            let dst_idx = ((y * dst_w + x) * 4) as usize;
            if src_idx + 3 < input.len() && dst_idx + 3 < output.len() {
                output[dst_idx..dst_idx + 4].copy_from_slice(&input[src_idx..src_idx + 4]);
            }
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_sheet_build() {
        let mut builder = SpriteSheetBuilder::new(64, 4);
        // Add 8 tiles
        for _ in 0..8 {
            let tile = vec![128u8; 64 * 64 * 4];
            builder.add_tile(&tile, 64, 64).unwrap();
        }
        let result = builder.build().unwrap();
        assert_eq!(result.thumb_count, 8);
        assert!(result.data.len() > 0);
        assert_eq!(result.tile_size, 64);
    }
}
