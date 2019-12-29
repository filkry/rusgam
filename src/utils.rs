pub fn align_up(size: usize, align: usize) -> usize {
    if size % align == 0 {
        return size;
    }

    let result = ((size / align) + 1) * align;

    assert!(result >= size);
    assert!(result % align == 0);

    result
}

pub fn clamp<T: Copy + PartialOrd<T>>(val: T, min: T, max: T) -> T {
    if val < min {
        return min;
    }
    else if val > max {
        return max;
    }

    return val;
}