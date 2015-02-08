.. _network_testing:

===============
Network Testing
===============

Usually vagga runs processes in host network namespace. But there is a mode
for network testing.


Overview
========

For testing complex networks we leverage ``!Supervise`` type of commands to
run multiple nodes. But we also need a way to setup network. What we need in
particular:

1. The IPs should be hard-coded (i.e. checked in into version control)
2. Multiple different projects running simultaneously (and multiple instances
   of same project as a special case of it)
3. Containers should be able to access internet if needed

So we use "double-bridging" to get this working, as illustrated below:

.. image:: double_bridging_vagga_networks.png


The :ref:`network_setup` section describes how to setup a gateway in
the host system, and :ref:`container_setup` section describes how
to configure containers in ``vagga.yaml``. And
:ref:`network_partitioning` section describes how to implement tests
which break network and create network partitions of various kinds.


.. _network_setup:

Setup
=====

Unfortunately we can't setup network in fully non-privileged way. So you need
to do some preliminary setup. To setup a bridge run::

    vagga _create_netns

Running this will show what commands are going to run::

    We will run network setup commands with sudo.
    You may need to enter your password.

    The following commands will be run:
        sudo 'ip' 'link' 'add' 'vagga_guest' 'type' 'veth' 'peer' 'name' 'vagga'
        sudo 'ip' 'link' 'set' 'vagga_guest' 'netns' '16508'
        sudo 'ip' 'addr' 'add' '172.18.255.1/30' 'dev' 'vagga'
        sudo 'sysctl' 'net.ipv4.conf.vagga.route_localnet=1'
        sudo 'mount' '--bind' '/proc/16508/ns/net' '/run/user/1000/vagga/netns'
        sudo 'mount' '--bind' '/proc/16508/ns/user' '/run/user/1000/vagga/userns'

    The following iptables rules will be established:
        ["-I", "INPUT", "-i", "vagga", "-d", "127.0.0.1", "-j", "ACCEPT"]
        ["-t", "nat", "-I", "PREROUTING", "-p", "tcp", "-i", "vagga", "-d", "172.18.255.1", "--dport", "53", "-j", "DNAT", "--to-destination", "127.0.0.1"]
        ["-t", "nat", "-I", "PREROUTING", "-p", "udp", "-i", "vagga", "-d", "172.18.255.1", "--dport", "53", "-j", "DNAT", "--to-destination", "127.0.0.1"]
        ["-t", "nat", "-A", "POSTROUTING", "-s", "172.18.255.0/30", "-j", "MASQUERADE"]

Then immediatelly the commands are run, this will probably request your
password by sudo command. The ``iptables`` commands may depend on DNS server
settings in your ``resolv.conf``.

.. note:: you can't just copy these commands and run (or push exact these
   commands to ``/etc/sudoers``), merely because the pid of the process in
   mount commands is different each time.

You may see the commands that will be run without running them with
``--dry-run`` option::

    vagga _create_netns --dry-run

To destroy the created network you can run::

    vagga _destroy_netns

This uses ``sudo`` too

.. warning:: if you have ``172.18.0.0/16`` network attached to your machine,
   the ``_create_netns`` and ``_destroy_netns`` may break that network. We will
   allow to customize the network in future versions of vagga.




.. _container_setup:

Containers
==========

# TBD


.. _network_partitioning:

Partitioning
============

# TBD


----

There is an article_ on how the network interface was designed
and why.

.. _article: https://medium.com/@paulcolomiets/evaluating-mesos-4a08f85473fb
