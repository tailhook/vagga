use std::os::getenv;
use std::io::{stdout, stderr};
use std::collections::HashSet;

use argparse::{ArgumentParser, List};

use config::Config;
use config::command::{Networking};
use config::command::{main};



pub struct Graph {
    pub drop_pairs: Vec<(String, String)>,
    pub isolate: Vec<String>,
}

pub fn full_mesh_cmd(_config: &Config, args: Vec<String>)
    -> Result<Graph, Result<int, String>>
{
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Returns network back to full connectivity");
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Err(Ok(0)),
            Err(x) => {
                return Err(Ok(x));
            }
        }
    }
    return Ok(Graph { drop_pairs: vec!(), isolate: vec!() });
}

pub fn disjoint_graph_cmd(config: &Config, args: Vec<String>)
    -> Result<Graph, Result<int, String>>
{
    let mut nodes: Vec<String> = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Splits graph into few disjoint clusters. Each node must be
            specified exactly once. Clusters are separated by double-dash.
            ");
        ap.refer(&mut nodes)
            .add_argument("node", box List::<String>, r#"
                List of nodes separated in clusters by "--"
                "#);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Err(Ok(0)),
            Err(x) => {
                return Err(Ok(x));
            }
        }
    }
    let cmd = try!(getenv("VAGGA_COMMAND")
        .and_then(|cmd| config.commands.find(&cmd))
        .ok_or(Err(format!("This command is supposed to be run inside \
                        container started by vagga !Supervise command"))));
    let sup = match cmd {
        &main::Supervise(ref sup) => sup,
        _ => return Err(Err(format!("This command is supposed to be run \
                inside container started by vagga !Supervise command"))),
    };
    let mut visited = HashSet::new();
    let mut clusters = vec!();
    let mut cluster: Vec<String> = vec!();
    for v in nodes.iter() {
        if v.as_slice() == "--" {
            if cluster.len() > 0 {
                clusters.push(cluster);
                cluster = vec!();
            }
            continue;
        }
        let node = try!(sup.children.find(v)
            .ok_or(Err(format!("Node {} does not exists", v))));
        if let Some(netw) = node.network() {
            let ref ip = netw.ip;
            if ip.as_slice() == "172.18.0.254" {
                return Err(Err(format!(
                    "Node {} is bridge and must not be used", v)));
            }
            cluster.push(ip.to_string());
            if !visited.insert(ip.to_string()) {
                return Err(Err(format!("Duplicate node {} (or it's IP)", v)));
            }
        } else {
            return Err(Err(format!("Node {} has no network", v)));
        }
    }
    if cluster.len() > 0 {
        clusters.push(cluster);
    }

    let mut pairs = HashSet::new();
    for i in visited.iter() {
        for j in visited.iter() {
            pairs.insert((i.clone(), j.clone()));
        }
    }
    for cluster in clusters.iter() {
        for i in cluster.iter() {
            for j in cluster.iter() {
                pairs.remove(&(i.to_string(), j.to_string()));
            }
        }
    }
    return Ok(Graph {
        drop_pairs: pairs.into_iter().collect(),
        isolate: vec!(),
        });
}

pub fn split_graph_cmd(config: &Config, args: Vec<String>)
    -> Result<Graph, Result<int, String>>
{
    let mut nodes: Vec<String> = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Splits graph into few clusters. Each node might participate in
            multiple clusters. Nodes which are not specified are isolated
            from all others.
            ");
        ap.refer(&mut nodes)
            .add_argument("node", box List::<String>, r#"
                List of nodes separated in clusters by "--"
                "#);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Err(Ok(0)),
            Err(x) => {
                return Err(Ok(x));
            }
        }
    }
    let cmd = try!(getenv("VAGGA_COMMAND")
        .and_then(|cmd| config.commands.find(&cmd))
        .ok_or(Err(format!("This command is supposed to be run inside \
                        container started by vagga !Supervise command"))));
    let sup = match cmd {
        &main::Supervise(ref sup) => sup,
        _ => return Err(Err(format!("This command is supposed to be run \
                inside container started by vagga !Supervise command"))),
    };
    let mut isolate: HashSet<String> = sup.children.iter()
        .filter(|&(_, cfg)| cfg.network().is_some())
        .map(|(_, cfg)| cfg.network().unwrap().ip.clone())
        .collect();
    let mut visited = HashSet::new();
    let mut clusters = vec!();
    let mut cluster: Vec<String> = vec!();
    for v in nodes.iter() {
        if v.as_slice() == "--" {
            if cluster.len() > 0 {
                clusters.push(cluster);
                cluster = vec!();
            }
            continue;
        }
        let node = try!(sup.children.find(v)
            .ok_or(Err(format!("Node {} does not exists", v))));
        if let Some(netw) = node.network() {
            let ref ip = netw.ip;
            if ip.as_slice() == "172.18.0.254" {
                return Err(Err(format!(
                    "Node {} is bridge and must not be used", v)));
            }
            cluster.push(ip.to_string());
            visited.insert(ip.to_string());
            isolate.remove(ip);
        } else {
            return Err(Err(format!("Node {} has no network", v)));
        }
    }
    if cluster.len() > 0 {
        clusters.push(cluster);
    }

    let mut pairs = HashSet::new();
    for i in visited.iter() {
        for j in visited.iter() {
            pairs.insert((i.clone(), j.clone()));
        }
    }
    for cluster in clusters.iter() {
        for i in cluster.iter() {
            for j in cluster.iter() {
                pairs.remove(&(i.to_string(), j.to_string()));
            }
        }
    }
    return Ok(Graph {
        drop_pairs: pairs.into_iter().collect(),
        isolate: isolate.into_iter().collect(),
        });
}

pub fn isolate_graph_cmd(config: &Config, args: Vec<String>)
    -> Result<Graph, Result<int, String>>
{
    let mut nodes: Vec<String> = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Splits graph into few clusters. Each node might participate in
            multiple clusters. Nodes which are not specified are isolated
            from all others.
            ");
        ap.refer(&mut nodes)
            .add_argument("node", box List::<String>, r#"
                List of nodes separated in clusters by "--"
                "#);
        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(0) => return Err(Ok(0)),
            Err(x) => {
                return Err(Ok(x));
            }
        }
    }
    let cmd = try!(getenv("VAGGA_COMMAND")
        .and_then(|cmd| config.commands.find(&cmd))
        .ok_or(Err(format!("This command is supposed to be run inside \
                        container started by vagga !Supervise command"))));
    let sup = match cmd {
        &main::Supervise(ref sup) => sup,
        _ => return Err(Err(format!("This command is supposed to be run \
                inside container started by vagga !Supervise command"))),
    };
    let mut isolate = HashSet::new();
    for v in nodes.iter() {
        let node = try!(sup.children.find(v)
            .ok_or(Err(format!("Node {} does not exists", v))));
        if let Some(netw) = node.network() {
            let ref ip = netw.ip;
            if ip.as_slice() == "172.18.0.254" {
                return Err(Err(format!(
                    "Node {} is bridge and must not be used", v)));
            }
            isolate.insert(ip.to_string());
        } else {
            return Err(Err(format!("Node {} has no network", v)));
        }
    }
    return Ok(Graph {
        drop_pairs: vec!(),
        isolate: isolate.into_iter().collect(),
        });
}
