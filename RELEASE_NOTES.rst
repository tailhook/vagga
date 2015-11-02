=============
Release Notes
=============

Vagga 0.4.1
===========

:Release Date: future

* ``!Tar`` and ``!TarInstall`` commands now support unpacking local files (#81)
* Container build process now locked, which avoid failure with cryptic error
  message on simultaneous builds (#80)
* Add ``_pack_image`` command
* Upgrade rust to v1.4.0
* Renamed and fixed ``vagga_network`` command as ``vagga _network`` subcommand
* The pip cache is now namespaces over distro and version (was singleton)
* Vagga now cleans apt lists cache on failed ``apt-get update`` (#108)

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
