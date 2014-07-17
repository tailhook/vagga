use collections::treemap::TreeMap;
use collections::treemap::TreeSet;

use super::settings::Settings;
use cfg = super::config;

pub struct Container {
    pub name: String,
    pub fullname: String,
    pub default_command: Option<String>,
    pub wrapper_script: Option<String>,
    pub builder: String,
    pub parameters: TreeMap<String, String>,
    pub container_root: Option<Path>,
    pub environ_file: Option<String>,
    pub environ: TreeMap<String, String>,
}

pub struct Environ {
    pub vagga_path: Path,
    pub vagga_exe: Path,
    pub work_dir: Path,
    pub project_root: Path,
    pub local_vagga: Path,
    pub variables: Vec<String>,
    pub config: cfg::Config,
    pub settings: Settings,
}

fn subst_vars<'x>(src: &'x String, vars: &TreeMap<&'x str, &str>,
    used: &mut TreeSet<&'x str>)
    -> Result<String, String>
{
    let mut res = String::new();
    let mut off = 0;
    for item in regex!("@[a-zA-Z_-]+@").captures_iter(src.as_slice()) {
        let (start, end) = item.pos(0).unwrap();
        let var = src.as_slice().slice(start+1, end-1);
        res.push_str(src.as_slice().slice(off, start));
        res.push_str(match vars.find(&var) {
            Some(val) => val.as_slice(),
            None => return Err(format!("Variable {} not found", var)),
        });
        used.insert(var);
        off = end;
    }
    res.push_str(src.as_slice().slice_from(off));
    return Ok(res);
}

impl Environ {
    pub fn get_container<'x>(&'x self, name: &String)
        -> Result<Container, String>
    {
        let src = match self.config.containers.find(name) {
            Some(x) => x,
            None => return Err(format!("Can't find container {}", name)),
        };
        let mut vars = TreeMap::new();
        for (k, v) in self.config.variants.iter() {
            match v.default {
                Some(ref val) => {
                    vars.insert(k.as_slice(), val.as_slice());
                }
                None => {}
            }
        }
        for (k, v) in self.settings.variants.iter() {
            vars.insert(k.as_slice(), v.as_slice());
        }
        for pairstr in self.variables.iter() {
            let mut pair = pairstr.as_slice().splitn('=', 1);
            let key = pair.next();
            let value = pair.next();
            if key.is_none() || value.is_none() {
                return Err(format!("Wrong variant declaration {}", pairstr));
            };
            vars.insert(key.unwrap(), value.unwrap());
        }
        let mut used = TreeSet::new();
        let mut parameters: TreeMap<String, String> = TreeMap::new();
        for (k, v) in src.parameters.iter() {
            parameters.insert(k.clone(),
                try!(subst_vars(v, &vars, &mut used)));
        }
        let mut environ: TreeMap<String, String> = TreeMap::new();
        for (k, v) in src.environ.iter() {
            environ.insert(k.clone(),
                try!(subst_vars(v, &vars, &mut used)));
        }
        let mut container = Container {
            name: name.clone(),
            fullname: name.clone(),
            default_command: match src.default_command {
                None => None,
                Some(ref cmd) => Some(try!(subst_vars(cmd, &vars, &mut used))),
                },
            wrapper_script: match src.wrapper_script {
                None => None,
                Some(ref val) => Some(try!(subst_vars(val, &vars, &mut used))),
                },
            environ_file: match src.environ_file {
                None => None,
                Some(ref val) => Some(try!(subst_vars(val, &vars, &mut used))),
                },
            builder: src.builder.clone(),
            parameters: parameters,
            environ: environ,
            container_root: None,
        };
        for item in used.iter() {
            container.fullname.push_str("--");
            container.fullname.push_str(item.as_slice());
            container.fullname.push_str("-");
            container.fullname.push_str(vars.find(item).unwrap().as_slice());
        }
        return Ok(container);
    }
}
