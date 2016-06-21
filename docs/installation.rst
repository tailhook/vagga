.. highlight:: bash

.. _installation:

============
Installation
============


Binary Installation
===================

.. note:: If you're ubuntu user you should use package.
   See :ref:`instructions below<ubuntu>`.

Visit http://files.zerogw.com/vagga/latest.html to find out latest
tarball version. Then run the following::

    $ wget http://files.zerogw.com/vagga/vagga-0.6.1.tar.xz
    $ tar -xJf vagga-0.6.1.tar.xz
    $ cd vagga
    $ sudo ./install.sh

Or you may try more obscure way::

    $ curl http://files.zerogw.com/vagga/vagga-install.sh | sh


.. note:: Similarly we have a `-testing` variant of both ways:

    * http://files.zerogw.com/vagga/latest-testing.html

    .. code-block:: bash

       $ curl http://files.zerogw.com/vagga/vagga-install-testing.sh | sh


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

.. note:: If you are courageous enough, you may try to use ``vagga-testing``
   repository to get new versions faster::

       deb http://ubuntu.zerogw.com vagga-testing main

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

    $ echo 'deb http://ubuntu.zerogw.com vagga main' | sudo tee /etc/apt/sources.list.d/vagga.list
    $ sudo apt-get update
    $ sudo apt-get install vagga

If your ubuntu is older, or you upgraded it without recreating a user, you
need to fill in ``/etc/subuid`` and ``/etc/subgid``. Command should be similar
to the following::

    $ echo "$(id -un):100000:65536" | sudo tee /etc/subuid
    $ echo "$(id -un):100000:65536" | sudo tee /etc/subgid

Or alternatively you may edit files by hand.

Now your vagga is ready to go.


.. _archlinux:

Arch Linux
==============================================

Default Arch Linux kernel doesn't contain ``CONFIG_USER_NS=y`` in configuration, you can check it with::

    $ zgrep CONFIG_USER_NS /proc/config.gz

You may use binary package from authors of vagga, by adding the following
to ``/etc/pacman.conf``::

        [linux-user-ns]
        SigLevel = Never
        Server = http://files.zerogw.com/arch-kernel/$arch

.. note:: alternatively you may use a package from AUR::

    $ yaourt -S linux-user-ns-enabled


Package is based on ``core/linux`` package and differ only with
``CONFIG_USER_NS`` option.  After it's compiled, update your bootloader
config, for GRUB it's probably::

    grub-mkconfig -o /boot/grub/grub.cfg

.. warning:: After installing a custom kernel you need to rebuild all the
   custom kernel modules. This is usually achieved by installing ``*-dkms``
   variant of the package and ``systemctl enable dkms``. More about DKMS is
   in `Arch Linux wiki`__.

   __ https://wiki.archlinux.org/index.php/Dynamic_Kernel_Module_Support

Then **reboot your machine** and choose ``linux-user-ns-enabled`` kernel
at grub prompt. After boot, check it with ``uname -a`` (you should have
text ``linux-user-ns-enabled`` in the output).

.. note:: TODO how to make it default boot option in grub?

Installing vagga from binary archive using AUR package_ (please note that
vagga-bin located in new AUR4 repository so it should be activated in your
system)::

    $ yaourt -S vagga-bin

If your ``shadow`` package is older than ``4.1.5``, or you upgraded it without recreating a user, after installation you may need to fill in ``/etc/subuid`` and ``/etc/subgid``. You can check if you need it with::

    $ grep $(id -un) /etc/sub[ug]id

If output is empty, you have to modify these files. Command should be similar to the following::

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
