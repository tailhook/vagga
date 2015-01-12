use super::graphs::Graph;

use std::io::process::{Command, InheritFd, ExitStatus};


fn _rule<W: Writer, S:Str>(out: &mut W, data: S) -> Result<(), String> {
    debug!("Rule: {}", data.as_slice());
    (writeln!(out, "{}", data.as_slice()))
    .map_err(|e| format!("Error piping firewall rule {}: {}",
                         data.as_slice(), e))
}

pub fn apply_graph(graph: Graph) -> Result<(), String> {
    let mut cmd = Command::new("iptables-restore");
    cmd.stdout(InheritFd(1)).stderr(InheritFd(2));
    debug!("Running {}", cmd);
    let mut prc = try!(cmd.spawn()
        .map_err(|e| format!("Can't run iptables-restore: {}", e)));
    {
        let pipe = prc.stdin.as_mut().unwrap();

        try!(_rule(pipe, "*filter"));

        try!(_rule(pipe, ":INPUT ACCEPT [0:0]"));
        try!(_rule(pipe, ":FORWARD ACCEPT [0:0]"));
        try!(_rule(pipe, ":OUTPUT ACCEPT [0:0]"));

        for ip in graph.isolate.iter() {
            try!(_rule(pipe, format!("-A FORWARD -s {}/32 -j DROP", ip)));
            try!(_rule(pipe, format!("-A FORWARD -d {}/32 -j DROP", ip)));
        }

        for &(ref source, ref dest) in graph.drop_pairs.iter() {
            try!(_rule(pipe,
                format!("-A FORWARD -s {}/32 -d {}/32 -j DROP", source, dest)));
        }

        try!(_rule(pipe, "COMMIT"));
    }
    match prc.wait() {
        Ok(ExitStatus(0)) => Ok(()),
        e => Err(format!("Error running iptables-restore: {}", e)),
    }
}
