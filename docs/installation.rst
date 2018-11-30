.. highlight:: bash

.. _installation:

============
Installation
============


.. contents:: Contents
   :local:


Binary Installation
===================

.. note:: If you're ubuntu user you should use package.
   See :ref:`instructions below<ubuntu>`.

Visit https://files.zerogw.com/vagga/latest.html to find out latest
tarball version. Then run the following::

    $ wget https://files.zerogw.com/vagga/vagga-0.8.1.tar.xz
    $ tar -xJf vagga-0.8.1.tar.xz
    $ cd vagga
    $ sudo ./install.sh

Or you may try more obscure way::

    $ curl -sSf https://files.zerogw.com/vagga/vagga-install.sh | sh


.. note:: Similarly we have a `-testing` variant of both ways:

    * https://files.zerogw.com/vagga/latest-testing.html

    .. code-block:: bash

       $ curl -sSf https://files.zerogw.com/vagga/vagga-install-testing.sh | sh


Runtime Dependencies
====================

Vagga is compiled as static binary, so it doesn't have many runtime
dependencies. It does require user namespaces to be properly set up, which
allows Vagga to create and administer containers without having root privilege.
This is increasingly available in modern distributions but may need to be
enabled manually.

* the ``newuidmap``, ``newgidmap`` binaries are required (either from
  ``shadow`` or ``uidmap`` package)

* known exception for Arch Linux: ensure ``CONFIG_USER_NS=y`` enabled in kernel. Default kernel doesn't contain it, you can check it with::

    $ zgrep CONFIG_USER_NS /proc/config.gz

  See :ref:`archlinux`

* known exception for Debian and Fedora: some distributions disable
  unprivileged user namespaces by default. You can check with::

    $ sysctl kernel.unprivileged_userns_clone
    kernel.unprivileged_userns_clone = 1

  or you may get::

    $ sysctl kernel.unprivileged_userns_clone
    sysctl: cannot stat /proc/sys/kernel/unprivileged_userns_clone: No such file or directory

  **Either one** is a valid outcome.

  In case you've got ``kernel.unprivileged_userns_clone = 0``, use something
  along the lines of::

    $ sudo sysctl -w kernel.unprivileged_userns_clone=1
    kernel.unprivileged_userns_clone = 1
    # make available on reboot
    $ echo kernel.unprivileged_userns_clone=1 | \
        sudo tee /etc/sysctl.d/50-unprivleged-userns-clone.conf
    kernel.unprivileged_userns_clone=1

* ``/etc/subuid`` and ``/etc/subgid`` should be set up. Usually you need at
  least 65536 subusers. This will be setup automatically by ``useradd`` in new
  distributions.  See ``man subuid`` if not. To check::

    $ grep -w $(whoami) /etc/sub[ug]id
    /etc/subgid:<you>:689824:65536
    /etc/subuid:<you>:689824:65536

The only other optional dependency is ``iptables`` in case you will be doing
:doc:`network tolerance testing</network>`.

See instructions specific for your distribution below.

.. _ubuntu:

Ubuntu
======

To install from vagga's repository just add the following to ``sources.list``
(see actual command below)::

    deb [arch=amd64 trusted=yes] https://ubuntu.zerogw.com vagga main

The process of installation looks like the following:

.. code-block:: console

    $ echo 'deb [arch=amd64 trusted=yes] https://ubuntu.zerogw.com vagga main' | sudo tee /etc/apt/sources.list.d/vagga.list
    deb https://ubuntu.zerogw.com vagga main
    $ sudo apt-get update
    [.. snip ..]
    Get:10 https://ubuntu.zerogw.com vagga/main amd64 Packages [365 B]
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
    Get:1 https://ubuntu.zerogw.com/ vagga/main vagga amd64 0.1.0-2-g8b8c454-1 [873 kB]
    Fetched 873 kB in 2s (343 kB/s)
    Selecting previously unselected package vagga.
    (Reading database ... 60919 files and directories currently installed.)
    Preparing to unpack .../vagga_0.1.0-2-g8b8c454-1_amd64.deb ...
    Unpacking vagga (0.1.0-2-g8b8c454-1) ...
    Setting up vagga (0.1.0-2-g8b8c454-1) ...

Now vagga is ready to go.

.. note:: If you are courageous enough, you may try to use ``vagga-testing``
   repository to get new versions faster::

       deb [arch=amd64 trusted=yes] https://ubuntu.zerogw.com vagga-testing main

   It's build right from git "master" branch and we are trying to keep "master"
   branch stable.

Ubuntu: Old Releases (precise, 12.04)
=====================================

For old ubuntu you need `uidmap`. It has no dependencies. So if your
ubuntu release doesn't have `uidmap` package (as 12.04 does), just fetch it
from newer ubuntu release::

    $ wget http://gr.archive.ubuntu.com/ubuntu/pool/main/s/shadow/uidmap_4.1.5.1-1ubuntu9_amd64.deb
    $ sudo dpkg -i uidmap_4.1.5.1-1ubuntu9_amd64.deb

Then run same sequence of commands, you run for more recent releases:

.. code-block:: console

    $ echo 'deb [arch=amd64 trusted=yes] https://ubuntu.zerogw.com vagga main' | sudo tee /etc/apt/sources.list.d/vagga.list
    $ sudo apt-get update
    $ sudo apt-get install vagga

If your ubuntu is older, or you upgraded it without recreating a user, you
need to fill in ``/etc/subuid`` and ``/etc/subgid``. Command should be similar
to the following::

    $ echo "$(id -un):100000:65536" | sudo tee /etc/subuid
    $ echo "$(id -un):100000:65536" | sudo tee /etc/subgid

Or alternatively you may edit files by hand.

Now your vagga is ready to go.

.. _debian:

Debian 8
========

Install Vagga like in Ubuntu:

.. code-block:: console

    $ echo 'deb [arch=amd64 trusted=yes] https://ubuntu.zerogw.com vagga main' | sudo tee /etc/apt/sources.list.d/vagga.list
    $ sudo apt-get update
    $ sudo apt-get install vagga

Then fix runtime dependencies:

.. code-block:: console

    $ echo 'kernel.unprivileged_userns_clone = 1' | sudo tee --append /etc/sysctl.conf
    $ sudo sysctl -p

Now your vagga is ready to go.

.. _archlinux:

Arch Linux
==============================================

Since ``4.14.5-1`` Arch Linux kernel has enabled ``CONFIG_USER_NS`` option,
you can check it with::

    $ zgrep CONFIG_USER_NS /proc/config.gz

The only thing you should to do with new kernel is to turn on sysctl flag::

    sysctl kernel.unprivileged_userns_clone=1

To preserve the flag between reboots just execute::

    echo kernel.unprivileged_userns_clone=1 | sudo tee -a /etc/sysctl.d/99-sysctl.conf

Installing vagga from binary archive using AUR package_ (please note that
vagga-bin located in new AUR4 repository so it should be activated in your
system)::

    $ yaourt -S vagga-bin

If your ``shadow`` package is older than ``4.1.5``, or you upgraded it
without recreating a user, after installation you may need to fill
in ``/etc/subuid`` and ``/etc/subgid``. You can check if you need it with::

    $ grep $(id -un) /etc/sub[ug]id

If output is empty, you have to modify these files. Command should be similar
to the following::

    $ echo "$(id -un):100000:65536" | sudo tee -a /etc/subuid
    $ echo "$(id -un):100000:65536" | sudo tee -a /etc/subgid


.. _linux-user-ns-enabled: https://aur.archlinux.org/packages/linux-user-ns-enabled/
.. _package: https://aur.archlinux.org/packages/vagga-bin


Building From Source
====================

The recommended way to is to build with vagga. It's as easy as installing vagga
and running ``vagga make`` inside the the clone of a vagga repository.

There is also a ``vagga build-packages`` command which builds ubuntu and binary
package and puts them into ``dist/``.

To install run::

    $ make install

or just (in case you don't have ``make`` in host system)::

    $ ./install.sh

Both support ``PREFIX`` and ``DESTDIR`` environment variables.

You can also build vagga out-of-container by using rustup.rs. Make sure you
have the musl target installed::

    $ rustup target add x86_64-unknown-linux-musl

Also make sure you have musl-gcc in your path::

    $ which musl-gcc
    /usr/bin/musl-gcc

Then just build using cargo and the appropriate target::

    $ cargo build --target x86_64-unknown-linux-musl

Building From Source Using Docker
=================================

Clone vagga repository::

   $ git clone https://github.com/tailhook/vagga && cd vagga

Describe vagga version::

   $ git describe

Result would be something like ``v0.8.1-19-g372bded``

Compile vagga with dockerized rust::

   $ docker run --rm --user "$(id -u)":"$(id -g)" -e VAGGA_VERSION=v0.8.1-19-g372bded -v "$PWD":/usr/src/vagga -w /usr/src/vagga rust cargo build --release

Compiled binary is moved to ``/usr/local/bin``


OS X / Windows
==============

We have two proof of concept wrappers around vagga:

* vagga-docker_ which leverages docker for mac to run vagga on OS X
* vagga-box_ a wrapper around VirtualBox (tested on OS X only so far)

If you'd like something more stable, try:

* `vagrant-vagga <https://github.com/rrader/vagrant-vagga>`_ (recommended)
* `vagga-barge <https://github.com/ailispaw/vagga-barge>`_
* Or just your own vagrant config (but see `this FAQ entry`_)

.. _vagga-docker: https://github.com/tailhook/vagga-docker
.. _vagga-box: https://github.com/tailhook/vagga-box
.. _this faq entry: https://vagga.readthedocs.io/en/latest/errors.html#don-t-run-vagga-on-shared-folders

.. _raspbian:

Raspbian Stretch (Debian 9)
===========================

Either compile on Raspberry Pi (be patient as it needs quite a while; take care not to run out of memory):

.. code-block:: console

    $ git clone https://github.com/tailhook/vagga.git
    $ cd vagga
    $ VAGGA_VERSION=$(git describe) CFLAGS=-I/usr/include/arm-linux-musleabihf cargo build --target=arm-unknown-linux-musleabihf

Or cross compile (recommended):

.. code-block:: console

    $ git clone https://github.com/tailhook/vagga.git
    $ cd vagga
    $ vagga make-arm
    $ scp target/arm-unknown-linux-musleabihf/debug/vagga <user@pi>:<path to vagga repo>

Installation needs to be run from inside cloned vagga repo on Raspberry Pi.

.. code-block:: console

    $ ./fetch_binaries.sh armhf
    $ sudo ./install.sh
    $ vagga -V
    $ sudo apt install uidmap

Run container with Alpine should be fine on all Pi models whereas Ubuntu is only confirmed for "Pi 2 model B" https://wiki.ubuntu.com/ARM/RaspberryPi

To run container with Ubuntu add ubuntu-miror to your vagga settings file

.. code-block:: console

    $ echo 'ubuntu-mirror: http://ports.ubuntu.com/ubuntu-ports' >> ~/.vagga.yaml
    $ # if you get error because of failed 'apt-get update' try different mirror, e.g.
    $ echo 'ubuntu-mirror: http://ftp.tu-chemnitz.de/pub/linux/ubuntu-ports/' >> ~/.vagga.yaml

In your vagga.yaml select proper architecture::

       setup:
       - !UbuntuRelease { codename: xenial, arch: armhf }
