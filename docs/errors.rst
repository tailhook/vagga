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

You should also check ``/etc/subgid``, add presumably the same contents to
``/etc/subgid`` (In subgid file the first field still contains your user name
not a group name).

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

Most distributions (known: Nix, Arch Linux, Fedora) have binaries as
part of "shadow" package, so have them installed on every system.

.. _root:

You should not run vagga as root
--------------------------------

Well, sometimes users get some ``permission denied`` errors and try to run vagga
with sudo. Running as root is **never** an answer.

Here is a quick check list on permission checks:

* Check owner (and permission bits) of ``.vagga`` subdirectory if it exists,
  otherwise the directory where ``vagga.yaml`` is (project dir). In case you
  have already run vagga as root just do ``sudo rm -rf .vagga``
* :ref:`subuid`
* :ref:`uidmap`
* Check ``uname -r`` to have version of ``3.9`` or greater
* Check ``sysctl kernel.unprivileged_userns_clone`` the setting must either
  *not exist* at all or have value of ``1``
* Check ``zgrep CONFIG_USER_NS /proc/config.gz`` or
  ``grep CONFIG_USER_NS "/boot/config-`uname -r`"`` (ubuntu)
  the setting should equal to ``y``

The error message might look like::

    You should not run vagga as root (see http://bit.ly/err_root)

Or it might look like a warning::

    WARN:vagga::launcher: You are running vagga as a user different from the owner of project directory. You may not have needed permissions (see http://bit.ly/err_root)

Both show that you don't run vagga with the user that owns the project.
The legitimate reasons to run vagga as root are:

* If you run vagga in container (i.e. in vagga itself) and the root is not a
  real root
* If your project dir is owned by root (for whatever crazy reason)

Both cases should inhibit the warning automatically, but as a last resort
you may try ``vagga --ignore-owner-check``. If you have good case where this
works, please file an issue and we might make the check better.

