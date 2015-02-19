=============
Release Notes
=============


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
