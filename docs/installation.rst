.. _installation:

============
Installation
============

Runtime Dependencies
====================

Vagga is compiled as static binary, so it doesn't have too much runtime
dependencies. It only requires user namespaces properly set up.
Usually you just need to install package (or compile) and run vagga.
See one of the following sections for specific instructions.

But if you need to understand gory details, here is what vagga requires in
order to run:

* ``CONFIG_USER_NS=y`` enabled in kernel (enabled in many  distributions by
  default, with Archlinux being known exception)
* ``newuidmap``, ``newgidmap`` binaries (either from ``shadow`` or ``uidmap``
  package)
* ``/etc/subuid`` and ``/etc/subgid`` set up, usually you need at least 65536
  subusers (it's set automatically by ``useradd`` in new distributions,
  see ``man /etc/subuid`` if not)
* The ``iptables`` in case you will do network tolerance testing

Some distributions (at least Debian and Fedora) patch kernel to disable
unprivileged user namespaces with sysctl. So you might need to run (and add it
to system startup)::

    sysctl -w kernel.unprivileged_userns_clone=1

See instructions specific for your distribution below.


Building From Source
====================

.. note:: The recommended way to building vagga is to install the tool from
   packages (see :ref:`Ubuntu` below and then build vagga using vagga itself,
   the text below describes old-style build process)


Build-time dependencies:

* Rust_ compiler 1.0.0-alpha (``rustc``)
* ``make`` (probably gnu variant)
* ``git`` (when installing from git source)

Run-time dependencies (basically none):

* ``glibc`` (probably you have it)
* ``newuidmap/newgidmap`` binaries (in ubuntu is separate ``uidmap`` package,
  but it's a part of ``shadow`` which is installed everywhere)

Process is as simple as following::

    git submodule update --init
    make
    sudo make install PREFIX=/usr


.. _Rust: http://rust.org
.. _linux: http://kernel.org

.. _ubuntu:

Ubuntu
======

To install from vagga's repository just add the following to `sources.list`::

    deb http://ubuntu.zerogw.com vagga main

The process of installation looks like the following:

.. code-block:: console

    $ echo 'deb http://ubuntu.zerogw.com vagga main' | sudo tee /etc/apt/sources.list.d/vagga.list
    deb http://ubuntu.zerogw.com vagga main
    $ sudo apt-get update
    [.. snip ..]
    Get:10 http://ubuntu.zerogw.com vagga/main amd64 Packages [365 B]
    [.. snip ..]
    Fetched 9,215 kB in 17s (532 kB/s)
    Reading package lists... Done
    $ sudo apt-get install vagga
    Reading package lists... Done
    Building dependency tree
    Reading state information... Done
    The following NEW packages will be installed:
      vagga
    0 upgraded, 1 newly installed, 0 to remove and 113 not upgraded.
    Need to get 873 kB of archives.
    After this operation, 4,415 kB of additional disk space will be used.
    WARNING: The following packages cannot be authenticated!
      vagga
    Install these packages without verification? [y/N] y
    Get:1 http://ubuntu.zerogw.com/ vagga/main vagga amd64 0.1.0-2-g8b8c454-1 [873 kB]
    Fetched 873 kB in 2s (343 kB/s)
    Selecting previously unselected package vagga.
    (Reading database ... 60919 files and directories currently installed.)
    Preparing to unpack .../vagga_0.1.0-2-g8b8c454-1_amd64.deb ...
    Unpacking vagga (0.1.0-2-g8b8c454-1) ...
    Setting up vagga (0.1.0-2-g8b8c454-1) ...

Now vagga is ready to go.

Ubuntu: Old Releases (precise, 12.04)
=====================================

For old ubuntu you need `uidmap`. It has no dependencies. So if your
ubuntu release doesn't have `uidmap` package (as 12.04 does), just fetch it
from newer ubuntu release::

    wget http://gr.archive.ubuntu.com/ubuntu/pool/main/s/shadow/uidmap_4.1.5.1-1ubuntu9_amd64.deb
    sudo dpkg -i uidmap_4.1.5.1-1ubuntu9_amd64.deb

Then run same sequence of commands, you run for more recent releases:

.. code-block:: console

    $ echo 'deb http://ubuntu.zerogw.com vagga main' | sudo tee /etc/apt/sources.list.d/vagga.list
    $ sudo apt-get update
    $ sudo apt-get install vagga

If your ubuntu is older, or you upgraded it without recreating a user, you
need to fill in ``/etc/subuid`` and ``/etc/subgid``. Command should be similar
to the following::

    echo "$(id -un):100000:65536" | sudo tee /etc/subuid
    echo "$(id -un):100000:65536" | sudo tee /etc/subgid

Or alternatively you may edit files by hand.

Now your vagga is ready to go.


Ubuntu: Building From Source
============================

Until rust is stable and added to ubuntu repository you need to fetch it from
rust-lang.org::

    wget https://static.rust-lang.org/dist/rust-1.0.0-alpha-x86_64-unknown-linux-gnu.tar.gz
    tar -xf rust-1.0.0-alpha-x86_64-unknown-linux-gnu.tar.gz
    cd rust-1.0.0-alpha-x86_64-unknown-linux-gnu
    ./install.sh --prefix=/usr

Building vagga::

    git clone git://github.com/tailhook/vagga
    cd vagga
    git submodule update --init
    make

Installing::

    sudo make install PREFIX=/usr

For upgrading you may build vagga using vagga, just run the following in source
directory of vagga::

    vagga build-ubuntu-package

It will put ``*.deb`` file in current directory.


