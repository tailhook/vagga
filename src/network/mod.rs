use std::env;
use std::io::{stdout, stderr};
use std::process::exit;
use std::path::Path;

use argparse::{ArgumentParser, Store, List};

use config::Config;
use super::config::read_config;
use self::iptables::apply_graph;

mod graphs;
mod iptables;
mod run;


pub fn run(cmdline: Vec<String>) -> i32 {
    let mut kind = "".to_string();
    let mut args: Vec<String> = vec!();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("
            Split the network into partitions.

            All kinds of network splits are idempotent and
            atomic. I.e. it operates on the whole network (in fact whole
            firewall table) and result is always the same regardless of
            previous operations. It also doesn't isolate containers from
            the bridge-namespaced nodes and from the internet.
            ");
        ap.refer(&mut kind)
            .add_argument("kind", Store, r#"
                Kind of partitioning to do:
                "disjoint" -- divide into few non-intersecting networks,
                "split" -- divide into graph of networks that may have some
                    'bridge' nodes,
                "isolate" -- isolate individual node(s) from anything and from
                    each other,
                "run" -- run arbitrary command in node's network namespaces
                    still being in same mount (filesystem) namespace.
                "#);
        ap.refer(&mut args)
            .add_argument("node", List, "
                A node(s) to operate on. See help of specific command
                for details
                ");
        ap.stop_on_first_argument(true);
        ap.silence_double_dash(false);
        match ap.parse(cmdline, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => return x,
        }
    }


    // TODO(tailhook) read also config from /work/.vagga/vagga.yaml
    let cfg = read_config(&Path::new("/work/vagga.yaml")).ok()
        .expect("Error parsing configuration file");  // TODO

    args.insert(0, format!("vagga_network {}", kind));
    match run_command(&cfg, kind, args) {
        Ok(()) => 0,
        Err(Ok(x)) => x,
        Err(Err(e)) => {
            error!("{}", e);
            1
        }
    }
}

pub fn run_command(cfg: &Config, kind: String, args: Vec<String>)
    -> Result<(), Result<i32, String>>
{
    let graph = match &kind[..] {
        "fullmesh" => try!(graphs::full_mesh_cmd(&cfg, args)),
        "disjoint" => try!(graphs::disjoint_graph_cmd(&cfg, args)),
        "split" => try!(graphs::split_graph_cmd(&cfg, args)),
        "isolate" => try!(graphs::isolate_graph_cmd(&cfg, args)),
        "run" => {
            try!(run::run_command_cmd(&cfg, args));
            return Ok(());
        }
        _ => {
            return Err(Err(format!("Unknown command {}", kind)));
        }
    };
    try!(apply_graph(graph).map_err(Err));
    Ok(())
}

pub fn main() {
    exit(run(env::args().collect()));
}
