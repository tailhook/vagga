.. _builders:

==================
Container Builders
==================


Nix
===

Nix_ is currently the most featureful builder. Good support of nix is good
because it matches vagga concepts most closely. Note, that you don't need
to have nixos_ installed to use *nix* builder. You only need to have nix
package manager installed


Depenendencies
--------------

* ``nix`` package manager (commands ``nix-env``, ``nix-store``,
  ``nix-instantiate``)
* ``rsync``


Paramaters
----------

``config``
    (default: ``default.nix``) The ``.nix`` file to read configuration from.
    The path is relative to project root.

``attribute``
    (default: empty). The attribute to use for ``nix-instantiate -A``


Best Practices
--------------

Few tips:

* You almost certainly want to import ``nixpkgs``
* Use ``nixpkgs.buildEnv`` to build environment, so you can run use short
  paths for commands
* Low-level things like ``coreutils`` must be explicitly specified

For example here is how vagga documentation used to build:

.. code-block:: nix

    let
      pkgs = import <nixpkgs> { };
    in {
      sphinx = pkgs.buildEnv {
        name = "vagga-sphinx-env";
        paths = with pkgs; with pkgs.pythonPackages; [
          gnumake
          bash
          coreutils
          sphinx
        ];
      };
    }

And how it's used in ``vagga.yaml``:

.. code-block:: yaml

    containers:

      sphinx:
        builder: nix
        parameters:
          config: default.nix
          attribute: sphinx

.. _nix: https://nixos.org/nix/
.. _nixos: http://nixos.org


Archlinux
=========

Current archlinux_ builder can only setup packages from archlinux binary
repositories. In future we are going to implement building source packages with
makepkg in the container.

.. note:: There is no versioning support for this backend. This means that
   containers will be versioned barely by list of packages. This should be
   ok for most uses, but it means that builds on different machines and/or in
   different periouds will vary.


Dependencies
------------

* ``pacman``
* ``wget``
* ``lxc`` (``lxc-usernsexec`` command, will remove this dependency in future)


Parameters
----------

``packages``
    (default: ``base``) A space-separated list of packages to install. Members
    of this list might also be package groups or requirement specifications
    (e.g. ``shadow>=4.1``) that are supported by pacman on a command-line.

``pacman_conf``
    (defaults to vagga's builtin config) A path to customized ``pacman.conf``.
    The path is relative to project root.


Tips
----

Nothing is installed by default. So usually you need ``bash`` and ``coreutils``

For example here is how container for vagga docs might be built:

.. code-block:: yaml

  sphinx-arch:
    builder: arch
    parameters:
      packages: python-sphinx make coreutils bash


Debian-simple
=============

The ``debian_simple`` backend can be used to setup debian (or ubuntu or
probably any other debian derivative) by just unpacking ``deb`` files. No
``configure`` and ``install`` phases are run.

.. warning:: Given the complexity of debian packages and bad design of
   debootstrap we have not found a good way to install debian packages in a
   container (without root privileges). But also unlike in arch, many debian
   packages do some crazy things after unpacking, so many packages after
   unpacking do not work at all or have files located in unusual places.


Simple debian system setup:

.. code-block:: yaml

   sphinx:
     builder: debian_simple
     parameters:
       packages: python-sphinx,make

Simple ubuntu system setup:

.. code-block:: yaml

   builder: debian_simple
   parameters:
     repo: http://archive.ubuntu.com/ubuntu
     suite: trusty
     packages: python-sphinx,make


Dependencies
-----------

* ``debootstrap`` (and all of its depedencies)


Parameters
----------

``repo``
    Repository for the packages. ``http://http.debian.net/debian/`` for Debian
    and ``http://archive.ubuntu.com/ubuntu`` for ubuntu.

``suite``
    The suite to run for debian it may be a version of OS or some special value
    like ``sid`` or ``stable``. Refer to debootstrap documentation for more
    info.


``arch``
    Target architecture (default should work)

``packages``
    A comma-separated packages to install




.. _archlinux: http://archlinux.org
