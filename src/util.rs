pub fn shuffle_copy<T: Clone>(items: &[T]) -> Vec<T> {
    let mut out: Vec<T> = items.to_vec();
    shuffle_in_place(&mut out);
    out
}

pub fn shuffle_in_place<T>(items: &mut [T]) {
    let n = items.len();
    if n < 2 {
        return;
    }
    for i in (1..n).rev() {
        let j = random_usize(i + 1);
        items.swap(i, j);
    }
}

fn random_usize(upper_exclusive: usize) -> usize {
    let x = js_sys::Math::random();
    (x * upper_exclusive as f64).floor() as usize
}

pub fn pick_random(items: &[String]) -> Option<String> {
    if items.is_empty() {
        return None;
    }
    let i = random_usize(items.len());
    Some(items[i].clone())
}
