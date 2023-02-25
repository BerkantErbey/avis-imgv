# avis-imgv

avis-imgv is a fast, configurable and color managed image viewer built with Rust and [egui](https://github.com/emilk/egui). My goal was for it to be be fast and to be able to adapt to any kind of hardware power through user configuration.

As of now it's only been tested in Linux but I don't see why it wouldn't work in Windows/MacOS. Configuration and cache directories are obtained through the `directories` crate which is platform agnostic.

## Dependencies

Sqlite and exiftool are required.

## Build

Simply run:

`RUSTC_BOOTSTRAP="qcms" cargo build --release`

To install look at the `install.sh` script and adapt it to your situation. It's still in a rudimentary state and untested in more systems.

## Color Management

Color management is done through [qcms](https://github.com/FirefoxGraphics/qcms).

Currently avis-imgv is shipped with three(sRGB, Adobe RGB and Display P3) profiles. A profile is chosen based on the exiftool tag "Profile Description" through a `contains` function. This is pretty lax as we can match more specific profiles like `RT_sRGB` with srgb. Open to suggestions on this behaviour. If no profile is matched an extraction will be attempted although it isn't optimal for maximum performance. For this reason it is suggested opening a PR with additional profiles.

Output Profile is sRGB by default and only supports built in profiles. If you need extra profiles either open a PR or edit `icc.rs` and add whichever ones you need for your local builds. It can be configured in `config.yaml`.

sRGB and Adobe RGB(ClayRGB) were taken from [elles_icc_profiles](https://github.com/ellelstone/elles_icc_profiles).

## Planned Features

- Configurable shortcuts
- Right click context menu with actions based on user configuration. For example running a bash script to copy the image with xclip or wl-copy, add an xmp file with a rating or open raw file with the same name in darktable.
- Shortcuts with actions based on user configuration with the exact same premisse as the context menu.

## Configuration

Configuration file should be: `~/.config/avis-imgv/config.yaml`. An example is provided in the repo.

### General

Keys | Values | Default 
--- | --- | ---
limit_cached  | Maximum number of cached files metadata | 100000
output_icc_profile | Output icc profile | srgb 
text_scaling | Text Scaling | 1.25

### Gallery

Keys | Values | Default 
--- | --- | ---
loaded_images | Number of loaded images in each direction. Adjust based on how much RAM you want to use. | 5
should_wait | Should wait for image to finish loading before advancing to the next one | true
metadata_tags | Metadata visible in the Image Information side pannel(when opened) | Date/Time Original, Created Date, Camera Model Name, Lens Model, Focal Length, Aperture Value, Exposure Time, ISO, Image Size, Color Space, Directory
frame_size_relative_to_image | White frame size relative to smallest image side | 0.2

### Multi Gallery

Keys | Values | Default 
--- | --- | ---
images_per_row | How many images should be displayed per row | 3
preloaded_rows | How many off-screen rows in each direction should be loaded and remain in memory | 2
image_size | Max image size(largest side) for images. Downscale algorithm is Catmull-Rom which provides decent time saving and quality compared to Lanczos3 | 1000 (Good for a 4k screen with 3 images per row)
simultaneous_load | How many images should be allowed to load at the same time | 8 (Adjust according to core count or how much you want to work your PC)
margin_size | Margin size between images | 10.

## Shortcuts

### General

Key | Action
--- | --- 
Esc | Toggle between Single and Multi Gallery
Q | Exit

### Single Gallery

Key | Action
--- | --- 
F | Fit image to screen
G | Toggle a white frame around the image
Spacebar | Toggle Zoom
Ctrl+Scroll | Zoom image
Scroll | Next or Previous 
Arrow Keys | Next or Previous


### Multi Gallery
Key | Action
--- | ---
Spacebar | Scroll down
Double Click | Open Single gallery on selected image
