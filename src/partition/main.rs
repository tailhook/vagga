#![feature(phase, if_let)]

extern crate argparse;
#[phase(plugin, link)] extern crate log;

use std::os::set_exit_status;

use argparse::{ArgumentParser, Store, List};

use iptables::apply_graph;

mod graphs;
mod iptables;


fn run() -> Result<(), Result<int, String>> {
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
            .add_argument("kind", box Store::<String>, r#"
                Kind of partitioning to do:
                "disjoint" -- divide into few non-intersecting networks,
                "split" -- divide into graph of networks that may have some
                    'bridge' nodes,
                "isolate" -- isolate individual node(s) from anything and from
                    each other.

                "#);
        ap.refer(&mut args)
            .add_argument("node", box List::<String>, "
                A node(s) to operate on. See help of specific command
                for details
                ");
        ap.stop_on_first_argument(true);
        match ap.parse_args() {
            Ok(()) => {}
            Err(0) => return Err(Ok(0)),
            Err(x) => {
                return Err(Ok(x));
            }
        }
    }
    args.insert(0, format!("vagga_partition {}", kind));
    let graph = match kind.as_slice() {
        "disjoint" => try!(graphs::disjoint_graph_cmd(args)),
        "split" => try!(graphs::split_graph_cmd(args)),
        "isolate" => try!(graphs::isolate_graph_cmd(args)),
        _ => {
            return Err(Err(format!("Unknown command {}", kind)));
        }
    };
    try!(apply_graph(graph));
    Ok(())
}

fn main() {
    match run() {
        Ok(()) => {}
        Err(Ok(x)) => {
            set_exit_status(x);
        }
        Err(Err(e)) => {
            error!("{}", e);
            set_exit_status(1);
            return
        }
    }
}
