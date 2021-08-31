use super::*;

use arrayvec::{ArrayVec};

impl t12::SInputLayoutDesc {
    pub fn create(input_element_descs: &[t12::SInputElementDesc]) -> Self {
        let mut result = Self {
            input_element_descs: ArrayVec::new(),
            d3d_input_element_descs: ArrayVec::new(),
        };

        result
            .input_element_descs
            .try_extend_from_slice(input_element_descs)
            .unwrap();
        result
    }
}
