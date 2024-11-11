use crate::bindings::{hb_set_get_population, hb_set_next_many, hb_set_t};

/// Converts a HarfBuzz set to a Rust u32 array.
pub fn u32_array_from_hb_set(set_ptr: *mut hb_set_t) -> Vec<u32> {
    unsafe {
        let count = hb_set_get_population(set_ptr);
        let mut result = vec![0_u32; count as usize];
        hb_set_next_many(set_ptr, 0, result.as_mut_ptr() as _, count);
        result
    }
}
