=========
OverlayFS
=========

This page documents overlayfs_ support for vagga. This is currently a work
in progress.

Since *unprivileged* overlayfs is unsupported in mainline kernel, you may need
some setup. Anyway, **ubuntu**'s stock kernel has the patch applied.

The Plan
========

1. Make of use of overlayfs in :volume:`Snapshot` volume. This will be enabled
   by a volume-level setting initially. In perspective the setting will be
   default on systems that support it.
2. Use overlayfs for ``_run --writable`` and transient copies
3. Use overlayfs for :step:`Container` step. This will be enabled by a
   container-level setting. Which, presumably, will always be disabled by
   default.
4. Add ``vagga _build container --cache-each-step`` to ease debugging of
   container builds (actually to be able to continue failing build from any
   failed step)

Smaller things:

* ``vagga _check_overlayfs_support``

We need a little bit more explanation about why we would keep overlayfs
disabled by default. The first thing to know, is that while we will mount
overlays for filesystems inside the container, we can't mount overlays outside
of the container.

So we want to have first class IDE support by default (so you can point to one
folder for project dependencies, not variable list of layered folders)

For ``--cache-each-step`` the main reason is performance. From experience with
Docker_ we know that snapshotting each step is not zero-cost.

Setup
=====

This section describes quircks on variuos systems that are needed to enable
this feature.

To check this run::

    $ vagga _check_overayfs_support
    supported
    $ uname -r -v
    4.5.0 #1-NixOS SMP Mon Mar 14 04:28:54 UTC 2016

If first command reports ``supported`` please report your value of
``uname -rv`` so we can add it to the lists below.

The `original patch` made by Canonical's employee is just one line, and has
pretty extensive documentation about why it's safe enough.


Ubuntu
------

It works by default on Ubuntu_ trusty 14.04. It's reported successfully
on the following systems::

    3.19.0-42-generic #48~14.04.1-Ubuntu SMP Fri Dec 18 10:24:49 UTC 2015


Arch Linux
----------

Since you already use custom kernel, you just need another patch. If you
use the package recommended in `installation page<archlinux_>` your kernel
**already supports** overlayfs too.

The `AUR package`_ has he feature enabled too, this is were you can find
the PKGBUILD to build the kernel yourself.


NixOS
-----

On NixOS_ you need to add a patch and rebuild the kernel. Since the patch
is already in the nixos source tree, you need just the following in your
``/etc/nixos/configuration.nix``::

  nixpkgs.config.packageOverrides = pkgs: {
    linux_4_5 = pkgs.linux_4_5.override { kernelPatches = [
      pkgs.kernelPatches.ubuntu_unprivileged_overlayfs
    ]; };
  };

Adjust kernel version as needed.


.. _overlayfs: https://en.wikipedia.org/wiki/OverlayFS
.. _ubuntu: https://ubuntu.com
.. _nixos: https://nixos.org
.. _archlinux: https://archlinux.org
.. _AUR package: https://aur.archlinux.org/packages/linux-user-ns-enabled/
.. _original patch: http://people.canonical.com/~apw/lp1377025-utopic/0001-UBUNTU-SAUCE-Overlayfs-allow-unprivileged-mounts.patch
.. _docker: http://docker.com
