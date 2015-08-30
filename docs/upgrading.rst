=========
Upgrading
=========


Upgrading 0.2.x -> 0.3.x
========================

This upgrade should be seamless. The release is focused on migrating code
from pre-1.0 Rust to... well... rust 1.2.0.

Other aspect of code migration is that it uses ``musl`` libc. So building vagga
from sources is more complex now. (However it's as easy as previous version if
you build with vagga itself, except you need to wait until rust builds for the
first time).


Upgrading 0.1.x -> 0.2.x
========================

There are basically two things changed:

1. The way how containers (images) are built
2. Differentiation of commands

Building Images
---------------

Previously images was build by two parts: ``builder`` and ``provision``:

.. code-block:: yaml

  rust:
    builder: ubuntu
    parameters:
      repos: universe
      packages: make checkinstall wget git uidmap
    provision: |
      wget https://static.rust-lang.org/dist/rust-0.12.0-x86_64-unknown-linux-gnu.tar.gz
      tar -xf rust-0.12.0-x86_64-unknown-linux-gnu.tar.gz
      cd rust-0.12.0-x86_64-unknown-linux-gnu
      ./install.sh --prefix=/usr

Now we have a sequence of steps which perform work as a ``setup`` setting:

.. code-block:: yaml

  rust:
    setup:
    - !Ubuntu trusty
    - !UbuntuUniverse ~
    - !TarInstall
      url: http://static.rust-lang.org/dist/rust-1.0.0-alpha-x86_64-unknown-linux-gnu.tar.gz
      script: "./install.sh --prefix=/usr"
    - !Install [make, checkinstall, git, uidmap]
    - !Sh "echo Done"

Note the following things:

* Downloading and unpacking base os is just a step. Usually the first one.
* Steps are executed sequentially
* The amount of work at each step is different as well as different level of
  abstractions
* The ``provision`` thing may be split into several ``!Sh`` steps in new vagga

The description of each step is in :ref:`Reference <build_commands>`.

By default ``uids`` and ``gids`` are set to ``[0-65535]``. This default should
be used for all contianers unless you have specific needs.

The ``tmpfs-volumes`` key changed for the generic ``volumes`` key, see
:ref:`volumes` for more info.

The ``ensure-dirs`` feature is now achieved as ``- !EnsureDir dirname`` build
step.


Commands
--------

Previously type of :ref:`command<commands>` was differentiated by existence
of ``supervise`` and ``command``/``run`` key.

Now first kind of command is marked by ``!Command`` yaml tag. The ``command``
and ``run`` differentation is removed. When ``run`` is a list it's treated as
a command with arguments, if ``run`` is a string then it's run by shell.

The ``!Supervise`` command contains the processes to run in ``children`` key.

See :ref:`reference <commands>` for more info.


Missing Features
----------------

The following features of vagga 0.1 are missing in vagga 0.2. We expect
that they were used rarely of at all.

* Building images by host package manager (builders: debian-debootstrap,
  debian-simple, arch-simple). The feature is considered too hard to use and
  depends on the host system too much.

* Arch and Nix builders. Will be added later. We are not sure if we'll keep a
  way to use host-system nix to build nix container.

* Docker builder. It was simplistic and just PoC. The builder will be added
  later.

* Building images without ``uidmap`` and properly set ``/etc/subuid`` and
  ``/etc/subgid``. We believe that all systems having ``CONFIG_USER_NS``
  enabled have subuids either already set up or easy to do.

* The ``mutable-dirs`` settings. Will be replaced by better mechanism.


