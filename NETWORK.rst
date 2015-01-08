====================
Networking Internals
====================


Overall
=======

The connection scheme is following::

    host -- gateway -- bridge ×n -- guest ×n×m

Terms are used with the following meanings:

host
  Is a network on host system. We add a veth device (named "vagga") with
  a single address on it.

gateway
  Is a network that is created by ``_create_netns`` once. And is used by all
  containers. It has peer device of veth "vagga" named "vagga_guest". It
  also has a veth per each ``bridge`` process.

bridge
  A network namespace that is created per each vagga command (once for all
  subcommands of !Supervise command). It's there so that each ``bridge`` have
  it's own 172.18.0.0/24 network and they don't interfere each other.
  The ``×n`` above means there is *n* bridges for *n* simultaneous commands
  running.


guest
  A network namespace for each container running (i.e. each subcommand of
  vagga command). The guests of each bridge have same network. So in each
  vagga command same IPs 172.18.0.1, 172.18.0.2 may be used. But by proper
  namespacing and bridging they don't interfere each other. But subcommands
  of each command do see each other by using bridge, unless ``vagga_partition``
  is used.



Networks Used
=============

The hard-coded network and host addresses for now are:

* 172.18.255.1/30 (host) -- 172.18.255.2/30 (gateway) for host to gateway.
  The 172.18.255.1 is the only IP visible in the host system.
* 172.18.192.0 - 172.18.223.255 (or /19) -- for gateway to bridge IP addresses,
  they are generated automatically when command starts, and are visible only
  to bridge-hosted processes (i.e. one usually doing network tests). The /30
  networks are allocated, so up to 2048 simultaneous bridges (commands)
  are supported.
* 172.18.0.1 - 172.18.0.253 -- for guest addresses (bridge to guest network).
  They may be freely assigned by user. Simultaneous runs of vagga commands are
  isolated, as well as running vagga in multiple projects in parallel.
* 172.18.0.254 -- bridge ip address in

The bridge to guest network may be extended to 172.18.0.0 -- 172.18.127.255,
or /17 network, if we find out that 254 hosts is not enough for everyone.

We also plan to allow to override host ip address in settings in case users
have that IP/network used by something.


Network Utilities
=================

Host dependencies (used inside sudo):
* iptables
* ip (iproute2)
* mount

Usage for containers:
* ip (iproute2) -- used from host, because busybox doesn't support "veth"
* busybox brctl -- from busybox, so user don't need bridge-utils on host
* iptables -- used from host system (no busybox support at all)

(i.e. for this to work we setup network namespaces running in host system
mount namespace)


Network Split Emulation
=======================

Inside container requires:

* volume !VaggaBin -- so we may access ``vagga_partition`` without installing
  vagga inside container (not that it's complex, but so that we don't need to
  cope with version mismatches)
* environ PATH=/vagga/bin/directory:... -- so script can run vagga_partition
* iptables in PATH, needed by vagga_partition
