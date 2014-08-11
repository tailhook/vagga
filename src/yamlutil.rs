use collections::treemap::TreeMap;

use J = serialize::json;

use super::config::Range;

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

pub fn get_bool(json: &J::Json, key: &'static str) -> Option<bool> {
    return match json {
        &J::Object(ref dict) => match dict.find(&key.to_string()) {
            Some(&J::String(ref val)) => match val.as_slice() {
                "true"|"TRUE"|"True"|"yes"|"YES"|"Yes"|"y"|"Y" => Some(true),
                "false"|"FALSE"|"False"|"no"|"NO"|"No"|"n"|"N" => Some(false),
                ""|"~"|"null" => Some(false),
                _ => None,
            },
            Some(&J::Number(val)) => Some(val != 0.),
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

pub fn get_command(json: &J::Json, key: &'static str) -> Option<Vec<String>> {
    let mut res = Vec::new();
    let list = match json {
        &J::Object(ref dict) => match dict.find(&key.to_string()) {
            Some(&J::List(ref val)) => val,
            Some(&J::String(ref val)) =>
                return Some(vec!(val.clone())),
            Some(&J::Number(val)) =>
                return Some(vec!(val.to_str().to_string())),
            _ => return None,
        },
        _ => return None,
    };

    for item in list.iter() {
        match item {
            &J::String(ref val) => {
                res.push(val.clone());
            }
            _ => continue,  // TODO(tailhook) assert maybe?
        }
    }

    return Some(res);
}

pub fn get_ranges(json: &J::Json, key: &'static str) -> Vec<Range> {
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
            &J::Number(ref val) => {
                res.push(Range::new(*val as uint, *val as uint));
            }
            &J::String(ref val) => {
                match regex!(r"^(\d+)-(\d+)$").captures(val.as_slice()) {
                    Some(caps) => {
                        res.push(Range::new(
                            from_str(caps.at(1)).unwrap(),
                            from_str(caps.at(2)).unwrap()));
                    }
                    None => continue,  // TODO(tailhook) assert maybe?
                }
            }
            _ => continue,  // TODO(tailhook) assert maybe?
        }
    }

    return res;
}
