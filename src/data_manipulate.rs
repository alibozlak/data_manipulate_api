pub fn manipulate_datas_between_0_and_10(inputs : Vec<Vec<f64>>, outputs : Vec<f64>)
                                         -> (Vec<Vec<f64>>, Vec<f64>, Vec<usize>) {
    let m = inputs.len();
    let n = inputs[0].len();
    let mut result_inputs : Vec<Vec<f64>> = Vec::with_capacity(m);
    let mut result_outputs : Vec<f64> = Vec::with_capacity(m);
    let mut ten_power_ratios : Vec<usize> = vec![0; n + 1];

    for i in 0..m {
        let mut row : Vec<f64> = Vec::with_capacity(n);
        for j in 0..n {
            let (new_data, ratio) = find_ratio(inputs[i][j]);
            if i == 0 { ten_power_ratios[j] = ratio }
            row.push(new_data);
        }
        result_inputs.push(row);

        let (new_data, ratio) = find_ratio(outputs[i]);
        if i == 0 { ten_power_ratios[n] = ratio }
        result_outputs.push(new_data);
    }

    (result_inputs, result_outputs, ten_power_ratios)
}

fn find_ratio(data : f64) -> (f64, usize) {
    let mut ratio : usize = 0;
    let positive_data = data.abs();
    let mut data_s_string = positive_data.to_string();
    if let Some(index) = data_s_string.find(".") {
        data_s_string = data_s_string.split_at(index).0.to_string();
    }
    if data_s_string.len() > 1 { ratio = data_s_string.len() - 1; }

    (data * 10.0_f64.powi(-(ratio as i32)), ratio)
}