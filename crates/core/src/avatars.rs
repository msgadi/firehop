//! Filesystem [`AvatarStore`]: one normalised PNG per account.

use std::fs;
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat, ImageReader, Limits};

use crate::application::{AvatarError, AvatarStore};
use crate::domain::AccountId;

/// Stored avatars are this many pixels square: sharp at 44 px on 2.5× displays,
/// and small enough (a few KB) to ship inside every roster update.
const AVATAR_PX: u32 = 128;
/// Refuse source files above this size before decoding anything.
const MAX_SOURCE_BYTES: u64 = 20 * 1024 * 1024;
/// Refuse images whose decoded size would exceed this (guards against decompression bombs).
const MAX_DECODED_BYTES: u64 = 256 * 1024 * 1024;

pub struct FsAvatarStore {
    dir: PathBuf,
}

impl FsAvatarStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn path(&self, id: AccountId) -> PathBuf {
        self.dir.join(format!("{id}.png"))
    }
}

impl AvatarStore for FsAvatarStore {
    fn import(&self, id: AccountId, source: &Path) -> Result<String, AvatarError> {
        let png = normalise(source)?;
        fs::create_dir_all(&self.dir)
            .map_err(|err| failure("cannot create avatar folder", &err))?;
        let staging = self.path(id).with_extension("png.tmp");
        fs::write(&staging, &png)
            .and_then(|()| fs::rename(&staging, self.path(id)))
            .map_err(|err| failure("cannot save the photo", &err))?;
        Ok(data_url(&png))
    }

    fn load(&self, id: AccountId) -> Result<Option<String>, AvatarError> {
        match fs::read(self.path(id)) {
            Ok(png) => Ok(Some(data_url(&png))),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(failure("cannot read the photo", &err)),
        }
    }

    fn remove(&self, id: AccountId) -> Result<(), AvatarError> {
        match fs::remove_file(self.path(id)) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(failure("cannot delete the photo", &err)),
        }
    }
}

/// Decodes any supported image, crops the centre square and re-encodes it as a
/// small PNG. Re-encoding also strips metadata such as EXIF location.
fn normalise(source: &Path) -> Result<Vec<u8>, AvatarError> {
    let size = fs::metadata(source)
        .map_err(|err| failure("cannot open the file", &err))?
        .len();
    if size > MAX_SOURCE_BYTES {
        return Err(AvatarError("that image is larger than 20 MB".into()));
    }

    let mut reader = ImageReader::open(source)
        .map_err(|err| failure("cannot open the file", &err))?
        .with_guessed_format()
        .map_err(|err| failure("cannot read the file", &err))?;
    let mut limits = Limits::default();
    limits.max_alloc = Some(MAX_DECODED_BYTES);
    reader.limits(limits);
    let image = reader.decode().map_err(|_| {
        AvatarError("that file is not a supported image (PNG, JPEG, WebP or GIF)".into())
    })?;

    let mut png = Vec::new();
    square(&image)
        .resize_exact(AVATAR_PX, AVATAR_PX, FilterType::Lanczos3)
        .write_to(&mut Cursor::new(&mut png), ImageFormat::Png)
        .map_err(|err| AvatarError(format!("cannot convert the photo: {err}")))?;
    Ok(png)
}

fn square(image: &DynamicImage) -> DynamicImage {
    let side = image.width().min(image.height());
    let x = (image.width() - side) / 2;
    let y = (image.height() - side) / 2;
    image.crop_imm(x, y, side, side)
}

fn data_url(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", STANDARD.encode(png))
}

fn failure(what: &str, err: &io::Error) -> AvatarError {
    AvatarError(format!("{what}: {err}"))
}
