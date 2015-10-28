use std::io::Write;
use std::path::PathBuf;

use unshare::{Command, Stdio, Namespace};

use process_util::env_path_find;
use super::graphs::{Graph, NodeLinks};
use super::graphs::NodeLinks::{Full, Isolate, DropSome};
use super::super::container::nsutil::set_namespace;


fn _rule<W: Write, S:AsRef<str>>(out: &mut W, data: S) -> Result<(), String> {
    debug!("Rule: {}", data.as_ref());
    (writeln!(out, "{}", data.as_ref()))
    .map_err(|e| format!("Error piping firewall rule {:?}: {}",
        data.as_ref(), e))
}

pub fn apply_graph(graph: Graph) -> Result<(), String> {
    for (ip, node) in graph.nodes.iter() {
        try!(apply_node(ip, node));
    }
    Ok(())
}

fn apply_node(ip: &String, node: &NodeLinks) -> Result<(), String> {
    try!(set_namespace(
        format!("/tmp/vagga/namespaces/net.{}", ip), Namespace::Net)
        .map_err(|e| format!("Can't set namespace: {}", e)));
    let mut cmd = Command::new(env_path_find("iptables-restore")
        .unwrap_or(PathBuf::from("/sbin/iptables-restore")));
    cmd.stdin(Stdio::piped());
    debug!("Running {:?} for {}", cmd, ip);
    let mut prc = try!(cmd.spawn()
        .map_err(|e| format!("Can't run iptables-restore {:?}: {}", cmd, e)));
    {
        let ref mut pipe = prc.stdin.take().unwrap();

        try!(_rule(pipe, "*filter"));
        match *node {
            Full => {
                // Empty chains with ACCEPT default (except FORWARD, we expect
                // user doesn't use FORWARD, i.e. nested networks)
                try!(_rule(pipe, ":INPUT ACCEPT [0:0]"));
                try!(_rule(pipe, ":FORWARD DROP [0:0]"));
                try!(_rule(pipe, ":OUTPUT ACCEPT [0:0]"));
            }
            Isolate => {
                // The DROP default and accept packets to/from bridge as an
                // exception
                try!(_rule(pipe, ":INPUT DROP [0:0]"));
                try!(_rule(pipe, ":FORWARD DROP [0:0]"));
                try!(_rule(pipe, ":OUTPUT DROP [0:0]"));
                try!(_rule(pipe,
                    format!("-A INPUT -s 172.18.0.254/32 -j ACCEPT")));
                try!(_rule(pipe,
                    format!("-A OUTPUT -d 172.18.0.254/32 -j ACCEPT")));
            }
            DropSome(ref peers) => {
                // Empty chains with ACCEPT default (except FORWARD, we expect
                // user doesn't use FORWARD, i.e. nested networks)
                try!(_rule(pipe, ":INPUT ACCEPT [0:0]"));
                try!(_rule(pipe, ":FORWARD DROP [0:0]"));
                try!(_rule(pipe, ":OUTPUT ACCEPT [0:0]"));
                for peer in peers.iter() {
                    try!(_rule(pipe,
                        format!("-A INPUT -s {}/32 -d {}/32 -j DROP",
                        ip, peer)));
                }
            }
        }
        try!(_rule(pipe, "COMMIT"));
    }
    match prc.wait() {
        Ok(status) if status.success() => Ok(()),
        e => Err(format!("Error running iptables-restore: {:?}", e)),
    }
}
