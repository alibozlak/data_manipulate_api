/// Rescales every column into a single-digit range and reports the power of ten
/// each one was divided by.
///
/// The exponent is chosen **per column**, from the largest magnitude in it, and
/// then applied to every value in that column. Choosing it per value instead
/// reorders the column — `90` becomes `9.0` while `110` becomes `1.1` — which
/// destroys the very relationship a regression is meant to fit, and leaves
/// `ratios` describing only the row it was taken from.
///
/// Because one exponent covers a whole column, the transform is a plain linear
/// rescaling, so coefficients trained on the result map back exactly:
/// `a_j = a'_j * 10^(r_y - r_j)` and `b = b' * 10^r_y`.
pub fn manipulate_datas_between_0_and_10(inputs : Vec<Vec<f64>>, outputs : Vec<f64>)
                                         -> (Vec<Vec<f64>>, Vec<f64>, Vec<usize>) {
    let n = inputs[0].len();

    // One exponent per feature column, then one more for the outputs, which is
    // the layout `ratios` is documented to have.
    let mut ten_power_ratios : Vec<usize> = Vec::with_capacity(n + 1);
    for j in 0..n {
        let column_max = inputs.iter()
            .map(|sample| sample[j].abs())
            .fold(0.0_f64, f64::max);
        ten_power_ratios.push(exponent_for(column_max));
    }
    let outputs_max = outputs.iter().map(|output| output.abs()).fold(0.0_f64, f64::max);
    ten_power_ratios.push(exponent_for(outputs_max));

    let result_inputs : Vec<Vec<f64>> = inputs.iter()
        .map(|sample| (0..n).map(|j| scale(sample[j], ten_power_ratios[j])).collect())
        .collect();

    let result_outputs : Vec<f64> = outputs.iter()
        .map(|output| scale(*output, ten_power_ratios[n]))
        .collect();

    (result_inputs, result_outputs, ten_power_ratios)
}

/// The power of ten that brings `magnitude` below 10: the number of digits
/// before the decimal point, less one.
///
/// Anything under 1 gets an exponent of 0 rather than a negative one, so a
/// column that is already small is left alone instead of being scaled up.
fn exponent_for(magnitude : f64) -> usize {
    let digits_before_point = magnitude.abs().trunc().to_string();

    if digits_before_point.len() > 1 { digits_before_point.len() - 1 } else { 0 }
}

fn scale(value : f64, exponent : usize) -> f64 {
    value * 10.0_f64.powi(-(exponent as i32))
}
