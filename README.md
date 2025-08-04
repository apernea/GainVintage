# **GainVintage**

GainVintage is a precision-crafted analog-style saturation and tone-shaping plugin designed to bring the character of iconic vintage gear into your digital workflow. Drawing sonic inspiration from the Telefunken V76 tube preamp, SSL 4000E console channel, and tape saturation. GainVintage delivers punch, warmth, and harmonic richness with modern control.

Key Features:

**Mode Selector (TUBE / TAPE)**: Switch between creamy tube saturation and tape-style compression and soft clipping.

**Input Gain & Output Trim**: Shape your signal with analog-style headroom and gain-staging control.

**Harmonic Drive**: Add subtle to aggressive saturation, with harmonics modeled after real analog circuitry.

**VU Meter**: Monitor input/output levels with a vintage-style visual aesthetic.

Perfect for mixing engineers, beatmakers, and producers seeking that "glued together" analog feel with low CPU overhead and zero-latency processing, GainVintage enhances everything from drums and synths to vocals and full mixes.

## **IN ORDER TO RUN:**
**Prerequisites:**
1. Rust toolchain installed → https://rustup.rs/
2. Terminal (Command Line) access.

**Steps:**
1. Download the source code from GitHub (via ZIP or git clone).
2. Open Terminal and navigate to the downloaded folder: cd path/to/gain-vintage
3. Run the build command: cargo xtask bundle -p gain-vintage --release
4. After it completes, find the built plugin here:
    macOS: target/bundled/gain-vintage.vst3
    Windows: target\\bundled\\gain-vintage.vst3
5. Copy the .vst3 file to your system's VST3 folder:
    macOS: /Library/Audio/Plug-Ins/VST3/
    Windows: C:\\Program Files\\Common Files\\VST3\\
6. Open your DAW and rescan plugins.
7. Enjoy!