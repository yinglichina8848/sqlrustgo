use sqlrustgo_planner::Operator;

pub struct VectorizedComparison {
    pub left: Vec<i64>,
    pub right: Vec<i64>,
    pub result: Vec<bool>,
}

impl VectorizedComparison {
    pub fn new(left: Vec<i64>, right: Vec<i64>) -> Self {
        let size = left.len().min(right.len());
        Self {
            left,
            right,
            result: Vec::with_capacity(size),
        }
    }

    pub fn compare_eq(&mut self) {
        self.result.clear();
        for (&l, &r) in self.left.iter().zip(self.right.iter()) {
            self.result.push(l == r);
        }
    }

    pub fn compare_gt(&mut self) {
        self.result.clear();
        for (&l, &r) in self.left.iter().zip(self.right.iter()) {
            self.result.push(l > r);
        }
    }

    pub fn compare_lt(&mut self) {
        self.result.clear();
        for (&l, &r) in self.left.iter().zip(self.right.iter()) {
            self.result.push(l < r);
        }
    }
}

#[inline]
pub fn sum_i64_scalar(values: &[i64]) -> i64 {
    values.iter().sum()
}

#[inline]
pub fn sum_i64_simd_like(values: &[i64]) -> i64 {
    if values.len() < 8 {
        return values.iter().sum();
    }

    let chunk_size = 8;
    let num_chunks = values.len() / chunk_size;
    let remainder_start = num_chunks * chunk_size;

    let mut sums = [0i64; 8];
    for chunk_idx in 0..num_chunks {
        let start = chunk_idx * chunk_size;
        let end = start + chunk_size;
        for (i, &val) in values[start..end].iter().enumerate() {
            sums[i] = sums[i].wrapping_add(val);
        }
    }

    let mut total: i64 = 0;
    for s in &sums {
        total = total.wrapping_add(*s);
    }

    for &val in &values[remainder_start..] {
        total = total.wrapping_add(val);
    }

    total
}

#[inline]
pub fn avg_f64_scalar(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let sum: f64 = values.iter().sum();
    sum / values.len() as f64
}

#[inline]
pub fn avg_f64_kahan(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }

    let mut sum = 0.0;
    let mut c = 0.0;

    for &v in values {
        let y = v - c;
        let t = sum + y;
        c = (t - sum) - y;
        sum = t;
    }

    sum / values.len() as f64
}

pub fn filter_by_comparison(values: &[i64], threshold: i64, op: Operator) -> Vec<usize> {
    let mut indices = Vec::new();
    for (i, &val) in values.iter().enumerate() {
        let keep = match op {
            Operator::Gt => val > threshold,
            Operator::Lt => val < threshold,
            Operator::GtEq => val >= threshold,
            Operator::LtEq => val <= threshold,
            Operator::Eq => val == threshold,
            Operator::NotEq => val != threshold,
            _ => false,
        };
        if keep {
            indices.push(i);
        }
    }
    indices
}

pub fn batch_filter(values: &[i64], mask: &[bool]) -> Vec<i64> {
    values
        .iter()
        .zip(mask.iter())
        .filter(|(_, &m)| m)
        .map(|(&v, _)| v)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vectorized_comparison_eq() {
        let mut comp = VectorizedComparison::new(vec![1, 2, 3, 4], vec![1, 3, 3, 5]);
        comp.compare_eq();
        assert_eq!(comp.result, vec![true, false, true, false]);
    }

    #[test]
    fn test_vectorized_comparison_gt() {
        let mut comp = VectorizedComparison::new(vec![1, 2, 3, 4], vec![1, 3, 3, 5]);
        comp.compare_gt();
        assert_eq!(comp.result, vec![false, false, false, false]);
    }

    #[test]
    fn test_vectorized_comparison_lt() {
        let mut comp = VectorizedComparison::new(vec![1, 2, 3, 4], vec![1, 3, 3, 5]);
        comp.compare_lt();
        assert_eq!(comp.result, vec![false, true, false, true]);
    }

    #[test]
    fn test_sum_i64_scalar() {
        let values = vec![1i64, 2, 3, 4, 5];
        assert_eq!(sum_i64_scalar(&values), 15);
    }

    #[test]
    fn test_sum_i64_simd_like() {
        let values = vec![1i64, 2, 3, 4, 5];
        assert_eq!(sum_i64_simd_like(&values), 15);
    }

    #[test]
    fn test_sum_i64_simd_like_large() {
        let values: Vec<i64> = (0..1000).collect();
        assert_eq!(sum_i64_simd_like(&values), 499500);
    }

    #[test]
    fn test_avg_f64_scalar() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((avg_f64_scalar(&values) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_avg_f64_kahan() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!((avg_f64_kahan(&values) - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_filter_by_comparison() {
        let values = vec![1i64, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let indices = filter_by_comparison(&values, 5, Operator::Gt);
        assert_eq!(indices, vec![5, 6, 7, 8, 9]);

        let indices = filter_by_comparison(&values, 5, Operator::Lt);
        assert_eq!(indices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_batch_filter() {
        let values = vec![1i64, 2, 3, 4, 5];
        let mask = vec![true, false, true, false, true];
        let filtered = batch_filter(&values, &mask);
        assert_eq!(filtered, vec![1, 3, 5]);
    }
}
