//! The PixelCraft document model: pure data, no rendering, no UI.
//!
//! A [`Sprite`] is a grid of **layers × frames**. Where a layer meets a frame there may be a
//! [`Cel`]: an image placed at an offset on the canvas. Frames carry their own duration; tags name
//! frame ranges for animation; the palette belongs to the sprite.
//!
//! Layer order: `layers[0]` is the **bottom** layer, matching compositing order. UIs display the
//! list reversed.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]
#![forbid(unsafe_code)]

use std::sync::Arc;

use pixelcraft_color::{ColorMode, Palette};
use pixelcraft_geom::{Point, Size};
use pixelcraft_raster::{Image, RasterError};
use serde::{Deserialize, Serialize};

/// The most frames and layers a sprite may have; files and commands asking for more are refused.
pub const MAX_FRAMES: usize = 65_535;
pub const MAX_LAYERS: usize = 4_096;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CelError {
    #[error("there is no layer {0}")]
    NoLayer(usize),
    #[error("there is no frame {0}")]
    NoFrame(usize),
    #[error(transparent)]
    Raster(#[from] RasterError),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    Addition,
    Subtract,
    Difference,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layer {
    pub name: String,
    pub visible: bool,
    pub locked: bool,
    pub opacity: u8,
    pub blend: BlendMode,
    /// The background layer is opaque, can't move and sits at the bottom.
    pub background: bool,
}

impl Layer {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), visible: true, locked: false, opacity: 255, blend: BlendMode::Normal, background: false }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Frame {
    /// How long the frame shows during playback, in milliseconds.
    pub duration_ms: u32,
}

impl Default for Frame {
    fn default() -> Self {
        Self { duration_ms: 100 }
    }
}

/// An image on one layer in one frame.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cel {
    pub layer: usize,
    pub frame: usize,
    /// Top-left corner of the image on the canvas.
    pub position: Point,
    pub opacity: u8,
    pub image: Arc<Image>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnimationDirection {
    #[default]
    Forward,
    Reverse,
    PingPong,
}

/// A named, inclusive frame range (`from..=to`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub from: usize,
    pub to: usize,
    pub direction: AnimationDirection,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprite {
    pub size: Size,
    pub mode: ColorMode,
    pub palette: Palette,
    pub layers: Vec<Layer>,
    pub frames: Vec<Frame>,
    pub cels: Vec<Cel>,
    pub tags: Vec<Tag>,
}

impl Sprite {
    /// A new sprite with one layer, one frame and one empty cel covering the canvas.
    pub fn new(size: Size, mode: ColorMode) -> Result<Self, RasterError> {
        let image = Arc::new(Image::new(mode, size)?);
        Ok(Self {
            size,
            mode,
            palette: Palette::default_16(),
            layers: vec![Layer::new("Layer 1")],
            frames: vec![Frame::default()],
            cels: vec![Cel { layer: 0, frame: 0, position: Point::default(), opacity: 255, image }],
            tags: Vec::new(),
        })
    }

    pub fn cel(&self, layer: usize, frame: usize) -> Option<&Cel> {
        self.cels.iter().find(|c| c.layer == layer && c.frame == frame)
    }

    pub fn cel_mut(&mut self, layer: usize, frame: usize) -> Option<&mut Cel> {
        self.cels.iter_mut().find(|c| c.layer == layer && c.frame == frame)
    }

    /// The cel at (`layer`, `frame`), creating an empty canvas-sized one first if that layer has
    /// no cel in that frame yet (a new layer starts empty, like the target app's).
    pub fn cel_or_new(&mut self, layer: usize, frame: usize) -> Result<&mut Cel, CelError> {
        if layer >= self.layers.len() {
            return Err(CelError::NoLayer(layer));
        }
        if frame >= self.frames.len() {
            return Err(CelError::NoFrame(frame));
        }
        if self.cel(layer, frame).is_none() {
            let image = Arc::new(Image::new(self.mode, self.size)?);
            self.cels.push(Cel { layer, frame, position: Point::default(), opacity: 255, image });
        }
        self.cels.iter_mut().find(|c| c.layer == layer && c.frame == frame).ok_or(CelError::NoLayer(layer))
    }

    /// Total length of one playback of every frame, in milliseconds.
    pub fn duration_ms(&self) -> u64 {
        self.frames.iter().map(|f| u64::from(f.duration_ms)).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sprite_has_one_layer_one_frame_one_cel() {
        let s = Sprite::new(Size::new(32, 32), ColorMode::Rgba).unwrap();
        assert_eq!((s.layers.len(), s.frames.len(), s.cels.len()), (1, 1, 1));
        assert!(s.cel(0, 0).is_some());
        assert!(s.cel(3, 7).is_none());
        assert_eq!(s.duration_ms(), 100);
    }

    #[test]
    fn painting_a_new_layer_creates_its_cel() {
        let mut s = Sprite::new(Size::new(4, 4), ColorMode::Rgba).unwrap();
        s.layers.push(Layer::new("Layer 2"));
        assert!(s.cel(1, 0).is_none());
        assert!(s.cel_or_new(1, 0).is_ok());
        assert!(s.cel(1, 0).is_some());
        assert_eq!(s.cels.len(), 2);
        assert!(s.cel_or_new(1, 0).is_ok(), "the second call reuses it");
        assert_eq!(s.cels.len(), 2);
        assert_eq!(s.cel_or_new(5, 0).err(), Some(CelError::NoLayer(5)));
        assert_eq!(s.cel_or_new(0, 3).err(), Some(CelError::NoFrame(3)));
    }

    #[test]
    fn empty_canvas_is_refused() {
        assert!(Sprite::new(Size::new(0, 0), ColorMode::Indexed).is_err());
    }
}
