=========
Upgrading
=========

Upgrading 0.6.x -> 0.7.0
========================

This release only introduces minor incompatibilities and also changes hashes
of the containers (so all containers will be rebuild after vagga upgrades).

* Py2/Py3Requirements now properly hashes files containing ``-r`` (basically
  includes). This means if you had previously ``!Depends`` commands for that
  files, you may now remove them. But it also means that included files
  should exist when running vagga (i.e. before containers are built).
* ``vagga _run`` now searches in the following precedence if no ``PATH`` was
  set in container
  ``/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin``.
  Previously the precedence was reversed. This may influence you if you have
  commands with the same names both in ``/usr`` and ``/usr/local``
* ``Copy`` and ``Depends`` do not respect file permissions. Most of the time
  this means that on machines with different ``umask`` you still have same
  container hash. But it also means that if you change permissions on the
  file container does not get rebuilt (executable bit is still versioned).
* ``!Snapshot`` respects the owner and permissions of the source directory
  rather than using defaults from tmpfs volume. We consider this a bugfix, but
  it may break some things if you relied on old behavior
* :ref:`Environment variable precedence <environment>` changed to be more
  intuitive
* ``resolv.conf`` and ``hosts`` files are replaced again after :step:`Tar`,
  :step:`Ubuntu`, :step:`Container`, :step:`SubConfig`. It's a bugfix in
  most cases (i.e. some stalled files may be unpacked/copied in old vagga).
  But it may clobber your files if you expected old behavior.
* ``eatmydata`` is enabled for built-in commands only, if you relied on
  fast fsyncs earlier, your builds may be slow. You may use
  ``!Env { LD_PRELOAD: "/usr/lib/x86_64-linux-gnu/libeatmydata.so" }`` to
  restore old behavior (for xenial, for other distros path may be different).
* Previously we have ignored the error when we couldn't remount root file
  system as read-only (e.g. on ``tmpfs`` or when otherwise some options like
  ``nosuid`` were enabled), this is no longer the case (we learned how to make
  those volumes readonly). In some scenarios it may mean that previously
  writable folders are now read-only.
* If you relied on a symlink to ``/tmp/vagga/hosts``, we have removed it
  because it was rarely useful and sometimes imposed issues (for example
  when ``/tmp`` is readonly). We are working on a more long term solution. In
  the meantime you must either rely on hosts from the host system (by default)
  or create a file yourself (luckily IP addresses are static so it's easy,
  although may be boring).



Upgrading 0.5.0 -> 0.6.0
========================

This release doesn't introduce any severe incompatibilities. The bump of
version is motivated mostly by the change of container hashes because of
refactoring internals.

Minor incompatibilities are:

* Vagga now uses images from partner-images.ubuntu.com rather
  than cdimage.ubuntu.com
* Vagga now uses single level of uid mappings and doesn't use the actual
  mapping as part of container hash. This allows to use ``mount`` in container
  more easily and also means we have reproducible containers hashes across
  machines
* ``!Copy`` command now uses paths inside the container as the ``source``,
  previously was inside the capsule (because of a mistake), however using
  source outside of the ``/work`` has not been documented
* Checksum checking in ``!Tar`` and ``!TarInstall`` now works (previously you
  could use an archive with wrong ``sha256`` parameter)
* Vagga now uses ``tar-rs`` library for unpacking archives instead of busybox,
  this may mean some features are new, and some archives could fail (please
  report if you find one)
* Vagga now runs ``id -u -n`` for finding out username, previously was using
  long names which aren't supported by some distributions (alpine == busybox).
* Commands with name starting with underscore are not listed in ``vagga``
  and ``vagga _list`` by default (like built-in ones)
* Ubuntu commands now use ``libeatmydata`` by default, which makes installing
  packages about 3x faster
* We remove ``/var/spool/rsyslog`` in ubuntu, this is only folder that makes
  issues when rsyncing image because of permissions (it's not useful in
  container anyway)
* Updated ``quire`` requires you need to write ``!*Unpack`` instead
  of ``!Unpack``
* Remove ``change-dir`` option from ``SubConfig`` that never worked and was
  never documented


Upgrading 0.4.1 -> 0.5.0
========================

This release doesn't introduce any severe incompatibilities. Except in the
networking support:

* Change gateway network from ``172.18.0.0/16`` to ``172.23.0.0/16``,
  hopefully this will have less collisions

The following are minor changes during the container build:

* The stdin redirected from ``/dev/null`` and stdout is redirected to stderr
  during the build. If you really need asking a user (which is an antipattern)
  you may open a ``/dev/tty``.
* The ``.vagga/.mnt`` is now unmounted during build (fixes bugs with bad tools)
* ``!Depends`` doesn't resolve symlinks but depends on the link itself
* ``!Remove`` removes files when encountered (previously removed only when
  container already built), also the command works with files (not only dirs)

The following are bugfixes in container runtime:

* The ``TERM`` and ``*_proxy`` env vars are now propagated for supervise
  commands in the same way as with normal commands (previously was absent)
* Pseudo-terminals in vagga containers now work
* Improved SIGINT handling, now Ctrl+C in interactive processes such as
  ``python`` (without arguments) works as expected
* The signal messages ("Received SIGINT...") are now printed into stderr rather
  than stdout (for ``!Supervise`` type of commands)
* Killing vagga supervise with TERM mistakenly reported SIGINT on exit, fixed

And the following changes the hash of containers (this should not cause a
headache, just will trigger a container rebuild):

* Add support for ``arch`` parameter in ``!UbuntuRelease`` this changes hash
  sum of all containers built using ``!UbuntuRelease``


See `Release Notes`_ and `Github <github_v0.5.0_>`_ for all changes.

.. _`github_v0.5.0`: https://github.com/tailhook/vagga/compare/v0.4.1...v0.5.0


Upgrading 0.4.0 -> 0.4.1
========================

This is minor release so it doesn't introduce any severe incompatibilities.
The pip cache in this release is namespaced over distro and version. So old
cache will be inactive now. And should be removed manually by cleaning
``.vagga/.cache/pip-cache`` directory. You may do that at any time

See `Release Notes`_ and `Github <github_v0.4.1_>`_ for all changes.

.. _`github_v0.4.1`: https://github.com/tailhook/vagga/compare/v0.4.0...v0.4.1


Upgrading 0.3.x -> 0.4.x
========================

The release is focused on migrating from small amount of C code to "unshare"
crate and many usability fixes, including ones which have small changes in
semantics of configuration. The most important changes:

* The ``!Sh`` command now runs shell with ``-ex`` this allows better error
  reporting (but may change semantics of script for some obscure cases)
* There is now :opt:`kill-unresponsive-after` setting for ``!Supervise``
  commands with default value of ``2``. This means that processes will shut
  down unconditionally two seconds after ``Ctrl+C``.

See `Release Notes`_ and `Github <github_v0.4.0_>`_ for all changes.

.. _`Release Notes`: https://github.com/tailhook/vagga/blob/master/RELEASE_NOTES.rst
.. _`github_v0.4.0`: https://github.com/tailhook/vagga/compare/v0.3.0...v0.4.0


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
and ``run`` differentiation is removed. When ``run`` is a list it's treated as
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


