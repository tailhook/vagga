use std::os::getenv;
use std::io::{stdout, stderr};
use std::mem::swap;
use std::collections::{HashSet, HashMap};

use argparse::{ArgumentParser, List};

use config::Config;
use config::command::{Networking};
use config::command::{MainCommand};
use self::NodeLinks::*;


#[derive(PartialEq)]
pub enum NodeLinks {
    Full,
    Isolate,
    DropSome(Vec<String>),
}

pub struct Graph {
    pub nodes: HashMap<String, NodeLinks>,
}

pub fn get_full_mesh(config: &Config)
    -> Result<(HashMap<String, String>, Graph), String>
{
    let cmd = try!(getenv("VAGGA_COMMAND")
        .and_then(|cmd| config.commands.get(&cmd))
        .ok_or(format!("This command is supposed to be run inside \
                        container started by vagga !Supervise command")));
    let sup = match cmd {
        &MainCommand::Supervise(ref sup) => sup,
        _ => return Err(format!("This command is supposed to be run \
                inside container started by vagga !Supervise command")),
    };

    Ok((
        sup.children.iter()
            .filter(|&(_, child)| child.network().is_some())
            .map(|(name, child)| (name.to_string(),
                                  child.network().unwrap().ip.to_string()))
            .collect(),
        Graph {
            nodes: sup.children.iter()
                .filter(|&(_, child)| child.network().is_some())
                .map(|(_, child)| (child.network().unwrap().ip.to_string(),
                                   Full))
                .collect(),
        },
    ))
}

pub fn full_mesh_cmd(config: &Config, args: Vec<String>)
    -> Result<Graph, Result<isize, String>>
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
    let (_ips, graph) = try!(get_full_mesh(config).map_err(Err));
    return Ok(graph);
}

pub fn disjoint_graph_cmd(config: &Config, args: Vec<String>)
    -> Result<Graph, Result<isize, String>>
{
    let mut nodes: Vec<String> = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Splits graph into few disjoint clusters. Each node must be
            specified exactly once. Clusters are separated by double-dash.
            ");
        ap.refer(&mut nodes)
            .add_argument("node", Box::new(List::<String>), r#"
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
    Ok(try!(_partition(config, nodes, false).map_err(Err)))
}

pub fn split_graph_cmd(config: &Config, args: Vec<String>)
    -> Result<Graph, Result<isize, String>>
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
            .add_argument("node", Box::new(List::<String>), r#"
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
    Ok(try!(_partition(config, nodes, false).map_err(Err)))
}

fn _partition(config: &Config, nodes: Vec<String>, check_all: bool)
    -> Result<Graph, String>
{
    let (ips, mut graph) = try!(get_full_mesh(config));
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
        let ip = try!(ips.get(v)
            .ok_or(format!("Node {} does not exists or has no IP", v)));
        cluster.push(ip.to_string());
        if !visited.insert(ip.to_string()) {
            return Err(format!("Duplicate node {} (or it's IP)", v));
        }
    }
    if cluster.len() > 0 {
        clusters.push(cluster);
    }
    if check_all {
        for (name, ip) in ips.iter() {
            if !visited.contains(ip) {
                return Err(format!("Node {} is missing. \
                    You may use 'split' command if you want to skip some nodes\
                    ", name));
            }
        }
    }

    let mut pairs = HashSet::new();
    for i in visited.iter() {
        for j in visited.iter() {
            if i != j {
                pairs.insert((i.clone(), j.clone()));
            }
        }
    }
    for cluster in clusters.iter() {
        for i in cluster.iter() {
            for j in cluster.iter() {
                if i != j {
                    pairs.remove(&(i.to_string(), j.to_string()));
                }
            }
        }
    }
    for (i, j) in pairs.into_iter() {
        let node = graph.nodes.get_mut(&i).unwrap();
        if *node == Full {
            *node = DropSome(ips.iter()
                .filter(|&(_, ip)| ip.as_slice() != j.as_slice())
                .map(|(_, ip)| ip.to_string())
                .collect())
        } else if let DropSome(ref mut items) = *node {
            let mut old_items = vec!();
            swap(&mut old_items, items);
            *items = old_items.into_iter()
                .filter(|ip| ip.as_slice() != j.as_slice())
                .collect();
        } else {
            unreachable!();
        }
    }
    return Ok(graph);
}

pub fn isolate_graph_cmd(config: &Config, args: Vec<String>)
    -> Result<Graph, Result<isize, String>>
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
            .add_argument("node", Box::new(List::<String>), r#"
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
    let (ips, mut graph) = try!(get_full_mesh(config).map_err(Err));
    for v in nodes.iter() {
        let ip = try!(ips.get(v)
            .ok_or(Err(format!("Node {} does not exists", v))));
        *graph.nodes.get_mut(ip).unwrap() = Isolate;
    }
    return Ok(graph);
}
