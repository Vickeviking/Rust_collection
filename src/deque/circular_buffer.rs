struct CircularBuffer<T> {
    data: Vec<T>,
    start: usize,
    end: usize,
}
