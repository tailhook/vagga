use std::io::{stdout, stderr};
use std::path::Path;

use argparse::{ArgumentParser, Store, List};

use crate::config::{Config, find_config_or_exit};

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
                    still being in same mount (filesystem) namespace,
                "fullmesh" -- restore network to full connectivity.
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

    let (cfg, _) = find_config_or_exit(&Path::new("/work"), false);

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
        "fullmesh" => graphs::full_mesh_cmd(&cfg, args)?,
        "disjoint" => graphs::disjoint_graph_cmd(&cfg, args)?,
        "split" => graphs::split_graph_cmd(&cfg, args)?,
        "isolate" => graphs::isolate_graph_cmd(&cfg, args)?,
        "run" => {
            run::run_command_cmd(&cfg, args)?;
            return Ok(());
        }
        _ => {
            return Err(Err(format!("Unknown command {}", kind)));
        }
    };
    apply_graph(graph).map_err(Err)?;
    Ok(())
}
