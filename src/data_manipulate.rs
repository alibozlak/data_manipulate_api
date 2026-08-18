pub fn manipulate_datas_between_0_and_10(inputs : Vec<Vec<f64>>, outputs : Vec<f64>)
                                         -> (Vec<Vec<f64>>, Vec<f64>, Vec<usize>) {
    let m = inputs.len();
    let n = inputs[0].len();
    let mut result_inputs : Vec<Vec<f64>> = vec![vec![0.0; n]; m];
    let mut result_outputs : Vec<f64> = vec![0.0; m];
    let mut ten_power_ratios : Vec<usize> = vec![0; n + 1];

    for i in 0..n {
        let (scaled_data, ratio) = find_column_ratio(inputs[0][i]);
        result_inputs[0][i] = scaled_data;
        ten_power_ratios[i] = ratio;
    }
    let (scaled_data, ratio) = find_column_ratio(outputs[0]);
    result_outputs[0] = scaled_data;
    ten_power_ratios[n] = ratio;

    for i in 1..m {
        for j in 0..n {
            result_inputs[i][j] = convert_data(inputs[i][j], ten_power_ratios[j]);
        }

        result_outputs[i] = convert_data(outputs[i], ten_power_ratios[n]);
    }

    (result_inputs, result_outputs, ten_power_ratios)
}

fn convert_data(data : f64, ten_power_ratio : usize) -> f64 {
    data * 10.0_f64.powi(-(ten_power_ratio as i32))
}

fn find_column_ratio(data : f64) -> (f64, usize) {
    let mut ratio : usize = 0;
    let positive_data = data.abs();
    let mut data_s_string = positive_data.to_string();
    if let Some(index) = data_s_string.find(".") {
        data_s_string = data_s_string.split_at(index).0.to_string();
    }
    if data_s_string.len() > 1 { ratio = data_s_string.len() - 1; }

    (data * 10.0_f64.powi(-(ratio as i32)), ratio)
}