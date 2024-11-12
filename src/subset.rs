use crate::bindings::{
    hb_set_add, hb_set_clear, hb_set_del, hb_set_invert, hb_set_t, hb_subset_input_create_or_fail,
    hb_subset_input_destroy, hb_subset_input_reference, hb_subset_input_set, hb_subset_input_t,
    hb_subset_input_unicode_set, hb_subset_or_fail, HB_SUBSET_SETS_DROP_TABLE_TAG,
    HB_SUBSET_SETS_LAYOUT_FEATURE_TAG, HB_SUBSET_SETS_LAYOUT_SCRIPT_TAG,
};
use crate::common::{HarfbuzzObject, Owned};
use crate::Face;
use std::marker::PhantomData;
use std::ptr::NonNull;

#[derive(Debug, PartialEq, Eq)]
pub struct HbSet {
    ptr: *mut hb_set_t,
}

impl HbSet {
    pub fn add_chars(&self, chars: &[u32]) {
        for char in chars {
            unsafe {
                hb_set_add(self.ptr, *char);
            }
        }
    }
    pub fn delete_chars(&self, chars: &[u32]) {
        for char in chars {
            unsafe {
                hb_set_del(self.ptr, *char);
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Subset<'a> {
    raw: NonNull<hb_subset_input_t>,
    input_unicode_set: HbSet,
    marker: PhantomData<&'a [u8]>,
}

impl<'a> Subset<'a> {
    pub fn new() -> Owned<Subset<'a>> {
        let ptr = unsafe { hb_subset_input_create_or_fail() };
        unsafe { Owned::from_raw(ptr) }
    }
    pub fn clear_drop_table(&self) {
        unsafe {
            hb_set_clear(hb_subset_input_set(
                self.raw.as_ptr(),
                HB_SUBSET_SETS_DROP_TABLE_TAG,
            ));
        }
    }
    pub fn adjust_layout(&self) {
        unsafe {
            for iterator in [
                HB_SUBSET_SETS_LAYOUT_FEATURE_TAG,
                HB_SUBSET_SETS_LAYOUT_SCRIPT_TAG,
            ] {
                // Do the equivalent of --font-features=*
                let layout_features = hb_subset_input_set(self.raw.as_ptr(), iterator);
                hb_set_clear(layout_features);
                hb_set_invert(layout_features);
            }
        }
    }

    pub fn run_subset(&self, face: Owned<Face<'_>>) -> Owned<Face<'_>> {
        let result_ptr = unsafe { hb_subset_or_fail(face.as_raw(), self.raw.as_ptr()) };
        let face = Face::from_ptr(result_ptr);
        face
    }
}

unsafe impl<'a> HarfbuzzObject for Subset<'a> {
    type Raw = hb_subset_input_t;

    unsafe fn from_raw(raw: *const hb_subset_input_t) -> Self {
        let input_unicode_set =
            unsafe { hb_subset_input_unicode_set(raw as *mut hb_subset_input_t) };
        Subset {
            raw: NonNull::new(raw as *mut _).unwrap(),
            input_unicode_set: HbSet {
                ptr: input_unicode_set,
            },
            marker: PhantomData,
        }
    }

    fn as_raw(&self) -> *mut Self::Raw {
        self.raw.as_ptr()
    }

    unsafe fn reference(&self) {
        hb_subset_input_reference(self.as_raw());
    }

    unsafe fn dereference(&self) {
        hb_subset_input_destroy(self.as_raw());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_subset_unicodes() {
        let path = "testfiles/SourceSansVariable-Roman.ttf";
        let face = Face::from_file(path, 0).unwrap();
        let subset = Subset::new();
        let chars: [u32; 3] = [32, 33, 34];
        subset.input_unicode_set.add_chars(&chars);

        subset.clear_drop_table();
        subset.adjust_layout();

        let result_face = subset.run_subset(face);

        // match unicode length
        let unicodes = result_face.collect_unicodes();
        for (i, &item) in chars.iter().enumerate() {
            assert_eq!(unicodes[i], item);
        }

        // match binary length
        let face_data = result_face.face_data();
        let binary_data = face_data.get_data();
        assert_eq!(binary_data.len(), 3100);
    }
}
