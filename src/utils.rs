pub fn align_up(size: usize, align: usize) -> usize {
    if size % align == 0 {
        return size;
    }

    let result = ((size / align) + 1) * align;

    assert!(result >= size);
    assert!(result % align == 0);

    result
}
