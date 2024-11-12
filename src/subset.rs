use crate::bindings::{
    hb_set_add, hb_set_del, hb_set_destroy, hb_set_t, hb_subset_input_create_or_fail,
    hb_subset_input_t, hb_subset_input_unicode_set, hb_subset_or_fail,
};
use crate::common::{HarfbuzzObject, Owned};
use crate::Face;
use std::marker::PhantomData;

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
impl Drop for HbSet {
    fn drop(&mut self) {
        unsafe { hb_set_destroy(self.ptr) }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Subset<'a> {
    raw: *mut hb_subset_input_t,
    input_unicode_set: HbSet,
    marker: PhantomData<&'a [u8]>,
}

impl<'a> Subset<'a> {
    pub fn new() -> Subset<'a> {
        let ptr = unsafe { hb_subset_input_create_or_fail() };
        let input_unicode_set = unsafe { hb_subset_input_unicode_set(ptr) };

        Subset {
            raw: ptr,
            input_unicode_set: HbSet {
                ptr: input_unicode_set,
            },
            marker: PhantomData,
        }
    }
    pub fn run_subset(&self, face: Owned<Face<'_>>) -> Owned<Face<'_>> {
        let result_ptr = unsafe { hb_subset_or_fail(face.as_raw(), self.raw) };
        let face = Face::from_ptr(result_ptr);
        face
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
        let result_face = subset.run_subset(face);
        let unicodes = result_face.collect_unicodes();
        assert_eq!(unicodes.len(), 3);

        for (i, &item) in chars.iter().enumerate() {
            assert_eq!(unicodes[i], item);
        }
    }
}
