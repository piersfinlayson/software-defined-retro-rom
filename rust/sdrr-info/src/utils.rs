// Copyright (C) 2025 Piers Finlayson <piers@piers.rocks>
//
// MIT License

pub fn add_commas(n: u64) -> String {
    let s = n.to_string();
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();

    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}
