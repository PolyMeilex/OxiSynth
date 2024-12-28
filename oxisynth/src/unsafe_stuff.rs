//! All the unsafe code gathered in a single place, to keep it enclosed

/// [i16] -> [u8] conversion
pub fn slice_i16_to_u8(slice: &[i16]) -> &[u8] {
    let len = std::mem::size_of_val(slice);
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, len) }
}

/// [i16] -> [u8] conversion
pub fn slice_i16_to_u8_mut(slice: &mut [i16]) -> &mut [u8] {
    let len = std::mem::size_of_val(slice);
    unsafe { std::slice::from_raw_parts_mut(slice.as_ptr() as *mut u8, len) }
}

/// [f32] -> [u8] conversion
#[cfg_attr(not(test), allow(dead_code))]
pub fn slice_f32_to_u8(slice: &[f32]) -> &[u8] {
    let len = std::mem::size_of_val(slice);
    unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, len) }
}

/// Write samples interleaved
#[cfg(feature = "i16-out")]
pub fn write_samples_interleaved(slice: &mut [i16], synth: &mut crate::Synth) {
    let len = slice.len() / 2;
    unsafe {
        // TODO: Use raw pointers for this
        //
        // This is most likely UB, even tho those pointer will never be acccesed at the same time,
        // the shear fact that we are creating double &mut to the same ptr might cause the compiler
        // to miss-optimise something
        let ptr = slice as *mut [i16];
        synth.write_i16(len, &mut *ptr, 0, 2, &mut *ptr, 1, 2)
    }
}
