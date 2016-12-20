=============
Release Notes
=============

Vagga 0.7.0
===========

:Release Date: future

* Added support of custom command-line options with help for all kinds of
  commands
* Added ``!Persistent`` volumes (#204)
* Added ``isolate-network`` option for commands and command-line option
* Added ``isolate-network`` option for ``!RunAs`` build step
* Py2/Py3Requirements now properly hashes files containing ``-r`` (basically
  includes)
* Added ``include-regex`` option for ``!Copy``
* ``!Snapshot`` root now preserves ownership and permissions of
  source directory
* ``!Depends`` now supports directories and ``include-regex``, being on par
  with ``!Copy`` command for features
* Added ``expect-inotify-limit`` option
* Fixed PHP integration commands for Ubuntu xenial and Alpine v3.4
* Multiple ``vagga _clean`` can now be run in single directory without
  issues (this is mostly useful in CI and scripts)
* Added a warning when ``vagga.yml`` (not ``yaml``) is present
* Fixes bug with wrong precedence of directories in ``PATH`` when running a
  command using ``vagga _run``
* Allow mounting files (not only dirs) as external volumes
* Added ``files`` option to ``!Tmpfs`` volume
* Added ``umask`` option to ``!Copy``
* ``!Copy`` and ``!Depends`` no longer respect mode for generating hashes,
  only executable bit is used
* Allow to copy from different container when using ``!Snapshot`` volume
* Added ``data-dirs`` option for container
* Added ``AlpineRepo`` and ``Repo`` commands
* Supports ``!*Include`` directive to compose multiple files
* Fixed environment variable precedence (#326)
* ``resolv.conf`` and ``hosts`` are propagated from host more reliably in
  case of using subcontainers
* Fixed ``external-user-id`` for ``!RunAs``
* ``vagga _list -A`` now includes hidden commands as expected
* ``ca-certificates`` are now added to ``BuildDeps`` whenever any build
  tools are installed (by ``Py2Install``, ``NpmInstall`` and similar commands)
* ``/run`` volume mounted by default has now mode ``0o755`` (had ``0o766`` by
  a mistake)
* Fixes bug with remounting to readonly on volumes that were previously
  mounted with ``nosuid``, ``nodev`` or few other options
* ``eatmydata`` is only enabled for ``!Install`` and ``!BuildDeps`` rather
  than everything because it conflicts with some other ``LD_PRELOAD``ed
  things (like faketime)
* Added ``vagga _clean --volumes`` and ``vagga _clean --unused-volumes``
* Implemented ``vagga _clean --everything`` (again after 0.2)
* Upgraded embedded alpine tools (apk-tools 2.6.7, busybox 1.24.2)
* Vagga does not output environment of a running command any more,
  use ``VAGGA_DEBUG_CMDENV`` to show
* Removed writing ``/tmp/vagga/hosts`` in ``!Supervise`` commands, it was
  rarely useful and never documented


Vagga 0.6.1
===========

:Release Date: 14.06.2016

* Blacklists some non-working alpine mirrors


Vagga 0.6.0
===========

:Release Date: 11.06.2016

* vagga uses rust 1.9 and ubuntu xenial for building
* Refactored internals to use traits for commands instead of large enum. This
  makes adding more commands much easier.
* Ubuntu images are now fetched from ``http://partner-images.ubuntu.com``
  rather than ``http://cdimage.ubuntu.com``
* vagga now uses single level of uid mappings and doesn't use the actual
  mapping as part of container hash. This allows to use ``mount`` in container
  more easily and also means we have reproducible containers hashes across
  machines
* ``!Copy``: fixed crash on absent directories, fix copying paths outside of
  the ``/work``
* Uses ``libmount`` for many mount operations (not all yet)
* Added ``keep-composer`` and ``vendor-dir`` options to ``!ComposerSettings``
* New command ``!Unzip`` similar to ``!Tar``
* Implement (optional) checksum checking in ``!Tar`` and ``!TarInstall``
* The ``minimum-vagga`` now works even when it doesn't know all the commands
  in the config (still YAML syntax must be correct)
* Add support for ``volumes`` in commands (not only in containers)
* Vagga now uses ``tar-rs`` library for unpacking archives instead of busybox,
  this may mean some features are new, and some archives could fail (please
  report if you find one)
* Add ``!Container`` volume type, which allows to mount other container as a
  volume, mostly useful for deployment tools
* Vagga now runs ``id -u -n`` for finding out username, previously was using
  long names which aren't supported by some distributions (alpine == busybox)
* Root user may now run vagga without ``/etc/subuid`` this makes container in
  container scenario easier
* Failed remount read-only is now a warning, this has two implications: you can
  run vagga on tmpfs and in this case your root image is writable
* Add ``vagga -m`` which allows to run multiple vagga commands in sequence
* Add ``prerequsites`` option, which allows to run sequences of commands in
  different containers
* Add ``pass-tcp-port`` which allows to test systemd-like socket activation and
  other scenarios that need passing tcp socket as file descriptor
* Add ``image-cache-url`` option which allows to fetch cached image from
  somewhere instead of building it locally
* ``!Tar`` command now supports getting tar from ``/volumes/``
* Add ``!RunAs`` command which allows to get rid of ``sudo`` and ``su`` in
  build steps
* Add ``--at-least`` option for ``vagga _clean --unused``
* ``!Build`` command can copy file (previoulsy could only directory)
* Add ``build-lock-wait`` setting to allow simultaneous builds of containers
* Package lists from ``apt-get`` are now cached for each distribution and
  doesn't fail on concurrent builds
* Add ``--allow-multiple`` option to ``_init_storage_dir``
* Commands with name starting with underscore are not listed in ``vagga``
  and ``vagga _list`` by default (like built-in ones)
* Ubuntu commands now use ``libeatmydata`` by default, which makes installing
  packages about 3x faster
* We remove ``/var/spool/rsyslog`` in ubuntu, this is only folder that makes
  issues when rsyncing image because of permissions (it's not useful in
  container anyway)
* ``BuildDeps`` now don't try to ``apt-mark`` in subcontainer
* Updated ``quire`` requires you need to write ``!*Unpack`` instead
  of ``!Unpack``
* Remove ``change-dir`` option from ``SubConfig`` that never worked and was
  never documented


Vagga 0.5.0
===========

:Release Date: 03.04.2016

* ``!Depends`` doesn't resolve symlinks but depends on the link itself
* Pseudo-terminals in vagga containers now work
* ``!Remove`` removes files when encountered (previously removed only when
  container already built), also the command works with files (not only dirs)
* Add ``!Shapshot`, ``!Empty``, ``!BindRO`` volume types
* Add ``external-volumes`` setting, which allows to mount directories outside
  of the project dir
* Add ``minimum-vagga`` option, which hints user which version they should use
* Implement  ``!Build``, ``!Download``, ``!Copy`` build steps
* Add ``_init_storage_dir`` builtin command
* Add ``vagga _clean --unused`` mode of operation which is superior
  to ``--old``
* Allow to customize python and nodejs versions for ``Py*`` and ``Npm*`` steps
* Fix various bugs in networking implementation
* Add shell autocomplete (bash included, zsh can be configured)
* The ``.vagga/.mnt`` is now unmounted during build (fixes bugs with bad tools)
* Improved SIGINT handling, now Ctrl+C in interactive processes such as
  ``python`` (without arguments) works as expected
* The signal messages ("Received SIGINT...") are now printed into stderr rather
  than stdout (for ``!Supervise`` type of commands)
* Killing vagga supervise with TERM mistakenly reported SIGINT on exit, fixed
* Signal SIGQUIT is now correctly propagated
* Add PHP/Composer support
* Add Ruby/Bundler support
* Add support for ``arch`` parameter in ``!UbuntuRelease`` this changes hash
  sum of all containers built using ``!UbuntuRelease``
* The stdin redirected from ``/dev/null`` and stdout is redirected to stderr
  during the build
* You can now filter commands in supervise by tags
* Change gateway network from ``172.18.0.0/16`` to ``172.23.0.0/16``,
  hopefully this will have less collisions
* The ``TERM`` and ``*_proxy`` env vars are now propagated for supervise
  commands in the same way as with normal commands (previously was absent)
* Implemented shared image cache via ``_push_image`` command
  and ``image-cache-url`` option


Vagga 0.4.1
===========

:Release Date: 03.11.2015

* ``!Tar`` and ``!TarInstall`` commands now support unpacking local files (#81)
* Container build process now locked, which avoid failure with cryptic error
  message on simultaneous builds (#80)
* Add ``_pack_image`` command
* Upgrade rust to v1.4.0
* Renamed and fixed ``vagga_network`` command as ``vagga _network`` subcommand
* The pip cache is now namespaced over distro and version (was singleton)
* Vagga now cleans apt lists cache on failed ``apt-get update`` (#108)
* Add ``UbuntuPPA`` and ``AptTrust`` build steps

Vagga 0.4.0
===========

:Release Date: 11.10.2015

* Vagga now uses "unshare" and "signal" crates for working with containers
* Signal handling is changed:

  * User visible changes: Ctrl+C doesn't sent twice to children (was
    rarely noticeable), Ctrl+/ reliably kills vagga and children
  * The only signal that is propagated by vagga to children is now SIGTERM
  * SIGINT is never propagate to children by vagga itself (because it's usually
    sent to process group anyway by Ctrl+C)
  * Other signals like SIGQUIT (SIGHUP, SIGUSR1, ...), are not captured by
    vagga, so they terminate vagga, resulting child processes are killed by OS
    by the KILL signal).
  * So if you want to send one of the signals except SIGTERM, send it to the
    specific process not to vagga

* Fix broken ``!Alpine``, which always installed latests known version of the
  distribution instead specified
* Add kill-unresponsive-after_ setting
* In ubuntu guests by default `/usr/bin/chfn` is symlinked to `/bin/true`, this
  prevents errors on some host systems (#52), this changes hash of the
  ``!UbuntuRelease`` step
* Fix ``--only`` and ``--exclude`` for supervision commands (was broken since
  0.2.0)
* Add ``--no-build`` and ``--no-version-check`` options
* Fixed ``epilog`` option
* Implement support of ``git+https`` and ``hg+https`` urls in python
  requirements (#58)
* Add support of `Py3Requirements`/`Py3Install` for alpine (v3.2 has python3)
* Mount `/dev/shm` by default (needed for ubuntu host, fixes #32)
* Implement forwarding proxy variables by default (#38)
* Run ``!Sh`` scripts with ``-ex`` options (#72)
* Implement ``subdirs`` key for ``!Tmpfs`` volume
* Support tilde-expansion in ``storage-dir`` and ``cache-dir`` settings
* The ``/etc/hosts`` file now copied inside the container at start (#39)

.. _kill-unresponsive-after: http://vagga.readthedocs.org/en/latest/commands.html#opt-kill-unresponsive-after


Vagga 0.3.0
===========

:Release Date: 30.08.2015

* !Tar command without subdir specified ignores invisible files and dirs
  (ones starting with dot `.`) to determine subdir.
* Vagga now list of packages and log of duration of each step at a container
  folder (e.g. ``.vagga/container_name/../timings.log``)
* Add ``!UbuntuRelease`` builder to build non-lts ubuntu
* Add ``!Git`` and ``!GitInstall`` commands to install from git repository
  (similar to ``!Tar`` and ``!TarInstall``)
* Add ``user-id`` and ``external-user-id`` settings
* Implement ``!SubConfig`` build step (very experimental)
* Add ``trusted-hosts`` to ``!PipConfig``
* Add ``timings.log`` and various package lists to the container metadata for
  easier troubleshooting
* Add ``BindRW`` subvolume type
* No longer clean ``/var/lib/apt`` by default (better for reusing containers)


Vagga 0.2.5
===========

:Release Date: 03.03.2015

* A quick bugfix release of NpmInstall command


Vagga 0.2.4
===========

:Release Date: 03.03.2015

* Implement support of ``https`` links for Tar, TarInstall commands
* The ``!Py*`` commands now download latest pip via `get-pip.py`_. This
  effectively means (a) that new features (like checkout a git subdirectory)
  works, (b) the version of pip is uniform across distributions and
  (c) installing dependencies to not interfere with pip dependencies on ubuntu
  (e.g. previously requests library where removed when removing build
  dependencies)
* Fix ``!CacheDirs`` command which was broken few versions ago
* Add ``!Text`` command for easier writing files into container (e.g. configs)

.. _get-pip.py: https://pip.pypa.io/en/latest/installing.html


Vagga 0.2.3
===========

:Release Date: 19.02.2015

* Reasonable error message when not enough uids available (#7)
* When running as root vagga now can use all available uids and doesn't require
  subuid/subgid files setup, mostly useful for container-in-container
  scenarios (#7)
* The ``VAGGAENV_*`` environment vars will now be propagated to containers with
  the prefix stripped
* vagga now supports ``--env`` and ``--use-env`` command-line switch to set
  envionment variable for child processes and to propagate a variable from
  parent (i.e. user's) environment
* Add ``!Container`` build command, which may be used to build on top of
  another container
* The ``vagga _run`` now works with relative commands
* Experimantal ``auto-clean`` option for containers
* Add ``node-legacy`` as dependency of ``!Npm`` for ubuntu (required for many
  scripts)


Vagga 0.2.2
===========

:Release Date: 14.02.2015

* Add ``_version_hash`` command, mostly for scripting
* No need for tilde or null after ``!UbuntuUniverse`` (and probably other cases)
* Fix permission of ubuntu ``policy-rc.d``, which fixes installing packages
  having a daemon that start on install
* Configure apt to always use ``--no-install-recommends`` in ubuntu
* Add ``-W`` flag to ``_run`` command, to run writable (copy of) container
* Ubuntu will automatically use nearest mirror and allow to customize mirror
  in personal settings


Vagga 0.2.1
===========

:Release Date: 12.02.2015

This release fixes small issues appeared right after release and adds python
requirements.txt support.

* ``make install`` did not install vagga's busybox, effectively making vagga
  work only from source folder
* Add Py2Requirements and Py3Requirements
  `commands <http://vagga.readthedocs.org/en/latest/build_commands.html#pyreq>`_
* Implement writing ``/etc/resolv.conf`` (previously worked only by the fact
  that libc tries 127.0.0.1 when the file is empty)
* Fix positional arguments for shell-wrapped commands


Vagga 0.2.0
===========


:Release Date: 11.02.2015

This is backwards-incompatible release of vagga. See Upgrading_. The need for
changes in configuration format is dictated by the following:

* Better isolation of build process from host system
* More flexible build steps (i.e. don't fall back to shell scripting for
  everything beyond "install this package")
* Caching for all downloads and packages systems (not only for OS-level
  packages but also for packages installed by pip and npm)
* Deep dependency tracking (in future version we will not only track
  changes of dependencies in ``vagga.yaml`` but also in ``requirements.txt``
  and ``package.json`` or whatever convention exists; it's partially possible
  using Depends_ build step)

More features:

* Built by Rust ``1.0.0-alpha``
* Includes experimental network_ `testing tools`_


There are `some features missing`_, but we believe it doesn't
affect a lot of users.


.. _Upgrading: http://vagga.readthedocs.org/en/latest/upgrading.html
.. _some features missing: http://vagga.readthedocs.org/en/latest/upgrading.html#missing-features
.. _Depends: http://vagga.readthedocs.org/en/latest/build_commands.html#depends
.. _network: http://vagga.readthedocs.org/en/latest/network.html
.. _testing tools: https://medium.com/@paulcolomiets/evaluating-mesos-4a08f85473fb
