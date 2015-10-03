======
Errors
======

The document describes errors when running vagga on various systems. The manual
only includes errors which need more detailed explanation and troubleshooting.
Most errors should be self-descriptive.

.. _subuid:

Could not read /etc/subuid or /etc/subgid
-----------------------------------------

The full error might look like::

    ERROR:vagga::container::uidmap: Error reading uidmap: Can't open /etc/subuid: No such file or directory (os error 2)
    WARN:vagga::container::uidmap: Could not read /etc/subuid or /etc/subgid (see http://bit.ly/err_subuid)
    error setting uid/gid mappings: Operation not permitted (os error 1)

This means there is no ``/etc/subuid`` file. It probably means you need to
create one. The recommended contents are following::

    your_user_name:100000:65536

----

You may get another similar error::

    ERROR:vagga::container::uidmap: Error reading uidmap: /etc/subuid:2: Bad syntax: "user:100000:100O"
    WARN:vagga::container::uidmap: Could not read /etc/subuid or /etc/subgid (see http://bit.ly/err_subuid)
    error setting uid/gid mappings: Operation not permitted (os error 1)

This means somebody has edited ``/etc/subuid`` and made an error. Just open
the file (note it's owned by root) and fix the issue (in the example the last
character should be zero, but it's a letter "O").

.. _uidmap:

Can't find newuidmap or newgidmap
---------------------------------

Full error usually looks like::

    WARN:vagga::process_util: Can't find `newuidmap` or `newuidmap` (see http://bit.ly/err_idmap)
    error setting uid/gid mappings: No such file or directory (os error 2)

There might be two reasons for this:

1. The binaries are not installed (see below)
2. The commands are not in ``PATH``

In the latter case you should fix your ``PATH``.

The packages for Ubuntu >= 14.04::

    $ sudo apt-get install uidmap

The Ubuntu 12.04 does not have the package. But you may use the package from
newer release (the following version works fine on 12.04)::

    $ wget http://gr.archive.ubuntu.com/ubuntu/pool/main/s/shadow/uidmap_4.1.5.1-1ubuntu9_amd64.deb
    $ sudo dpkg -i uidmap_4.1.5.1-1ubuntu9_amd64.deb

Most distributions (known: Nix, Archlinux, Fedora), does have binaries as
part of "shadow" package, so have them installed on every system.
