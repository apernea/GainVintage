//Trait `Plugin` is not implemented for `GainVintage` [E0277]
use gain_vintage::GainVintage;
use nih_plug::prelude::*;

fn main() {
    nih_export_standalone::<GainVintage>();
}