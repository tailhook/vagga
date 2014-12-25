============
Installation
============


Generic Installation
====================


Build-time dependencies:

* Rust_ compiler 0.12 (``rustc``)
* ``make`` (probably gnu variant)
* ``git`` (when installing from git source)

Run-time dependencies (basically none):

* ``glibc``
* ``uidmap``


.. note:: Vagga uses linux_ namespaces, so works on linux system only.


Process is as simple as following::

    git submodule update --init
    make
    sudo make install


.. _Rust: http://rust.org
.. _linux: http://kernel.org


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

For most packages you need `uidmap`. It has no dependencies. So if your
ubuntu release doesn't have `uidmap` package (as 12.04 does), just fetch it
from newer ubuntu release::

    wget http://gr.archive.ubuntu.com/ubuntu/pool/main/s/shadow/uidmap_4.1.5.1-1ubuntu9_amd64.deb
    sudo dpkg -i uidmap_4.1.5.1-1ubuntu9_amd64.deb

Then run same sequence of commands, you run for more recent releases:

.. code-block:: console

    $ echo 'deb http://ubuntu.zerogw.com vagga main' | sudo tee /etc/apt/sources.list.d/vagga.list
    $ sudo apt-get update
    $ sudo apt-get install vagga

You need to ignore error in ``apt-get update`` as it tries to fetch i386
version of index for some reason. Still it fetches needed ``amd64`` too.

If your ubuntu is older, or you upgraded it without recreating a user, you
need to fill in ``/etc/subuid`` and ``/etc/subgid``. Command should be similar
to the following::

    echo '$(id -un):100000:65536' | sudo tee /etc/subuid
    echo '$(id -un):100000:65536' | sudo tee /etc/subgid

Or alternatively you may edit files by hand.

Now your vagga is ready to go.


Ubuntu: Building From Source
============================

Unfortunately rust-0.12 PPA doesn't work. You need to setup rust from
binaries::

    wget https://static.rust-lang.org/dist/rust-0.12.0-x86_64-unknown-linux-gnu.tar.gz
    tar -xf rust-0.12.0-x86_64-unknown-linux-gnu.tar.gz
    cd rust-0.12.0-x86_64-unknown-linux-gnu
    ./install.sh --prefix=/usr

Building vagga::

    git clone git://github.com/tailhook/vagga
    cd vagga
    git submodule update --init
    make

Installing::

    sudo make install PREFIX=/usr


