use collections::treemap::TreeMap;

use J = serialize::json;

pub fn get_string(json: &J::Json, key: &'static str) -> Option<String> {
    return match json {
        &J::Object(ref dict) => match dict.find(&key.to_string()) {
            Some(&J::String(ref val)) => Some(val.clone()),
            Some(&J::Number(val)) => Some(val.to_str().to_string()),
            _ => None,
        },
        _ => None,
    }
}

pub fn get_dict(json: &J::Json, key: &'static str) -> TreeMap<String, String> {
    let mut res = TreeMap::new();
    let dict = match json {
        &J::Object(ref dict) => match dict.find(&key.to_string()) {
            Some(&J::Object(ref val)) => val,
            _ => return res,
        },
        _ => return res,
    };

    for (k, v) in dict.iter() {
        match v {
            &J::String(ref val) => {
                res.insert(k.clone(), val.clone());
            }
            &J::Number(val) => {
                res.insert(k.clone(), val.to_str().to_string());
            }
            _ => continue,  // TODO(tailhook) assert maybe?
        }
    }

    return res;
}

pub fn get_list(json: &J::Json, key: &'static str) -> Vec<String> {
    let mut res = Vec::new();
    let list = match json {
        &J::Object(ref dict) => match dict.find(&key.to_string()) {
            Some(&J::List(ref val)) => val,
            _ => return res,
        },
        _ => return res,
    };

    for item in list.iter() {
        match item {
            &J::String(ref val) => {
                res.push(val.clone());
            }
            _ => continue,  // TODO(tailhook) assert maybe?
        }
    }

    return res;
}
