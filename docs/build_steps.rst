.. highlight:: yaml

.. _build_steps:

===========
Build Steps
===========

This is work in progress reference of build steps. See :ref:`build_commands`
for help until this document is done. There is also an alphabetic
:ref:`genindex`

All of the following build steps may be used as an item in :opt:`setup`
setting.


Container Bootstrap
===================

Command that can be used to bootstrap a container (i.e. may work on top
of empty container):

* :step:`Alpine`
* :step:`Ubuntu`
* :step:`UbuntuRelease`
* :step:`SubConfig`
* :step:`Container`
* :step:`Tar`

Ubuntu Commands
===============

.. step:: Ubuntu

   Simple and straightforward way to install Ubuntu LTS release.

   Example::

       setup:
       - !Ubuntu trusty

   The value is single string having the codename of release ``trusty`` or
   ``precise`` known to work at the time of writing.

   The Ubuntu LTS images are updated on daily basis. But vagga downloads and
   caches the image. To update the image that was downloaded by vagga you need
   to clean the cache.

.. step:: UbuntuRelease

   This is more exensible but more cumbersome way to setup ubuntu (comparing
   to :step:`Ubuntu`). For example to install trusty you need::

   - !UbuntuRelease { version: 14.04 }

   But you can install non-lts version with this command::

   - !UbuntuRelease { version: 15.10 }

   All options:

   version
     The verison of ubuntu to install. This must be digital ``YY.MM`` form,
     not a code name. **Required**.

   keep-chfn-command
     (default ``false``) This may be set to ``true`` to enable
     ``/usr/bin/chfn`` command in the container. This often doesn't work on
     different host systems (see `#52
     <https://github.com/tailhook/vagga/issues/52>`_ as an example). The
     command is very rarely useful, so the option here is for completeness
     only.


.. step:: AptTrust

   This command fetches keys with ``apt-key`` and adds them to trusted keychain
   for package signatures. The following trusts a key for ``fkrull/deadsnakes``
   repository::

       - !AptTrust keys: [5BB92C09DB82666C]

   By default this uses ``keyserver.ubuntu.com``, but you can specify
   alternative::

       - !AptTrust
         server: hkp://pgp.mit.edu
         keys: 1572C52609D

   This is used to get rid of the error similar to the following::

        WARNING: The following packages cannot be authenticated!
          libpython3.5-minimal python3.5-minimal libpython3.5-stdlib python3.5
        E: There are problems and -y was used without --force-yes

   Options:

   server
     (default ``keyserver.ubuntu.com``) Server to fetch keys from. May be
     a hostname or ``hkp://hostname:port`` form. Or actu

   keys
     (default ``[]``) List of keys to fetch and add to trusted keyring. Keys
     can include full fingerprint or **suffix** of the fingerprint. The most
     common is 8 hex digits form.

.. step:: UbuntuRepo

   Adds arbitrary debian repo to ubuntu configuration. For example to add
   newer python::

       - !UbuntuRepo
         url: http://ppa.launchpad.net/fkrull/deadsnakes/ubuntu
         suite: trusty
         components: [main]
       - !Install [python3.5]

   See :step:`UbuntuPPA` for easier way for dealing specifically with PPAs.

   Options:

   url
     Url to the repository. **Required**.

   suite
     Suite of the repository. The common practice is that suite is named just
     like codename of the ubuntu release. For example ``trusty``. **Required**.

   components
     List of the components to fetch packages from. Common practice to have a
     ``main`` component. So usually this setting contains just single
     element ``components: [main]``. **Required**.

.. step:: UbuntuPPA

   A shortcut to :step:`UbuntuRepo` that adds named PPA. For example, the
   following::

       - !Ubuntu trusty
       - !AptTrust keys: [5BB92C09DB82666C]
       - !UbuntuPPA fkrull/deadsnakes
       - !Install [python3.5]

   Is equivalent to::

       - !Ubuntu trusty
       - !UbuntuRepo
         url: http://ppa.launchpad.net/fkrull/deadsnakes/ubuntu
         suite: trusty
         components: [main]
       - !Install [python3.5]

.. step:: UbuntuUniverse

   The singleton step. Just enables an "universe" repository::

   - !Ubuntu trusty
   - !UbuntuUniverse
   - !Install [checkinstall]


Alpine Commands
===============

.. step:: Alpine


Distribution Commands
=====================

These commands work for any linux distributions as long as distribution is
detected by vagga. Latter basically means you used :step:`Alpine`,
:step:`Ubuntu`, :step:`UbuntuRelease` in container config (or in parent
config if you use :step:`SubConfig` or :step:`Container`)

.. step:: Install

.. step:: BuildDeps


Generic Commands
================

.. step:: Sh

.. step:: Cmd

.. step:: Tar

   Unpacks Tar archive into container's filesystem.

   Example::

       - !Tar
         url: http://something.example.com/some-project-1.0.tar.gz
         path: /
         subdir: some-project-1.0

   Downloaded file is stored in the cache and reused indefinitely. It's
   expected that the new version of archive will have a new url. But
   occasionally you may need to clean the cache to get the file fetched again.

   url
     **Required**. The url or a path of the archive to fetch. If the url
     startswith dot ``.`` it's treated as a file name relative to the project
     directory. Otherwise it's a url of the file to download.
   path
     (default ``/``). Target path where archive should be unpacked to. By
     default it's a root of the filesystem.
   subdir
     Subdirectory inside the archive to extract. May be ``.`` to extract the
     root of the archive.

   **This command may be used to populate the container from scratch**

.. step:: TarInstall

   Similar to :step:`Tar` but unpacks archive into a temporary directory and
   runs installation script.

   Example::

       setup:
       - !TarInstall
         url: http://static.rust-lang.org/dist/rust-1.4.0-x86_64-unknown-linux-gnu.tar.gz
         script: ./install.sh --prefix=/usr


   url
     **Required**. The url or a path of the archive to fetch. If the url
     startswith dot ``.`` it's treated as a file name relative to the project
     directory. Otherwise it's a url of the file to download.

   subdir
     Subdirectory which command is run in. May be ``.`` to run command inside
     the root of the archive.

     The common case is having a single directory in the archive,
     and that directory is used as a working directory for script by default.

   script
     The command to use for installation of the archive. Default is effectively
     a ``./configure --prefix=/usr && make && make install``.

     The script is run with ``/bin/sh -exc``, to have better error hadling
     and display. Also this means that dash/bash-compatible shell should be
     installed in the previous steps under path ``/bin/sh``.

.. step:: Git

.. step:: GitInstall



Files and Directories
=====================

.. step:: Text

   Writes a number of text files into the container file system. Useful for
   wrinting short configuration files (use external files and file copy
   or symlinks for writing larger configs)

   Example::

       setup:
       - !Text
         /etc/locale.conf: |
            LANG=en_US.UTF-8
            LC_TIME=uk_UA.UTF-8

.. step:: Remove

   Remove file or a directory from the container and keep it clean on the end
   of container build. Useful for removing cache directories.

   This is also inherited by subcontainers. So if you know that some installer
   leaves temporary (or other unneeded files) after a build you may add this
   entry instead of using shell `rm` command. The `/tmp` directory is cleaned
   by default. But you may also add man pages which are not used in container.

   Example::

   setup:
   - !Remove /var/cache/something

   For directories consider use :step:`EmptyDir` if you need to keep cleaned
   directory in the container.

.. step:: EnsureDir

.. step:: EmptyDir

.. step:: CacheDirs


Meta Data
=========

.. step:: Env

.. step:: Depends


Sub-Containers
==============

.. step:: Container

.. step:: SubConfig


Node.JS Commands
================

.. step:: NpmInstall


Python Commands
===============

.. step:: PipConfig

   The directive configures various settings of pythonic commands below. The
   mostly used option is ``dependencies``::

       - !PipConfig
           dependencies: true
       - !Py3Install [flask]

   Most options directly correspond to the pip command line options so refer to
   `pip help`_ for more info.

   .. note:: every time :step:`PipConfig` is specified options are **replaced**
      rather than *augmented*. In other words if you start a block of pythonic
      commands with :step:`PipConfig` all subsequent commands will be executed
      with same options, no matter which :step:`PipConfig` settings was before.

   All options:

   dependencies
       (default ``false``) allow to install dependencies. If the option is
       ``false`` (by default) pip is run with ``pip --no-deps``

   index-urls
       (default ``[]``) List of indexes to search for packages. This
       corresponds to ``--index-url`` (for the first element) and
       ``--extra-index-url`` (for all subsequent elements) options on the
       pip command-line.

       When the list is empty (default) the ``pypi.python.org`` is used.

   find-links
       (default ``[]``) List of urls to html files to parse for links to
       packages for download.

   trusted-hosts
       (default ``[]``) List of hosts that are trusted to download packages
       from.

   cache-wheels
       (default ``true``) Cache wheels between different rebuilds of the
       container. The downloads are always cached. Only binary wheels are
       toggled with the option. It's useful to turn this off if you build
       many containers with different dependencies.

       Starting with vagga v0.4.1 cache is namespaced by linux distribution and
       version. It was single shared cache in vagga <= v0.4.0

   .. _pip help: https://pip.readthedocs.org/en/stable/reference/pip_install/


.. step:: Py2Install

.. step:: Py2Requirements

.. step:: Py3Install

.. step:: Py3Requirements

