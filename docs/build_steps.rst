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
     a hostname or ``hkp://hostname:port`` form.

   keys
     (default ``[]``) List of keys to fetch and add to trusted keyring. Keys
     can include full fingerprint or **suffix** of the fingerprint. The most
     common is the 8 hex digits form.

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
     Suite of the repository. The common practice is that the suite is named just
     like the codename of the ubuntu release. For example ``trusty``. **Required**.

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

::

   setup:
   - !Alpine v3.2


Distribution Commands
=====================

These commands work for any linux distributions as long as distribution is
detected by vagga. Latter basically means you used :step:`Alpine`,
:step:`Ubuntu`, :step:`UbuntuRelease` in container config (or in parent
config if you use :step:`SubConfig` or :step:`Container`)

.. step:: Install

::

    setup:
    - !Ubuntu trusty
    - !Install [gcc, gdb]        # On Ubuntu, equivalent to `apt-get install gcc gdb -y`
    - !Install [build-essential] # `apt-get install build-essential -y`
    # Note that `apt-get install` is run 2 times in this example


.. step:: BuildDeps

::

    setup:
    - !Ubuntu trusty
    - !BuildDeps [wget]
    - !Sh echo "We can use wget here, but no curl"
    - !BuildDeps [curl]
    - !Sh echo "We can use wget and curl here"
    # Container built. Now, everything in BuildDeps(wget and curl) is removed from the container.


Generic Commands
================

.. step:: Sh

    Runs arbitrary shell shell command, for example::

        - !Ubuntu trusty
        - !Sh "apt-get install -y package"

    If you have more than one-liner you may use YAMLy *literal* syntax for it::

        setup:
        - !Alpine v3.2
        - !Sh |
           if [ ! -z "$(which apk)" ] && [ ! -z "$(which lbu)" ]; then
             echo "Alpine"
           fi
        - !Sh echo "Finished building the Alpine container"

    .. warning:: To run ``!Sh`` you need ``/bin/sh`` in the container. See
       :step:`Cmd` for more generic command runner.

    .. note:: The ``!Sh`` command is run by ``/bin/sh -exc``. With the flags
       meaning ``-e`` -- exit if any command fails, ``-x`` -- print command
       before executing, ``-c`` -- execute command. You may undo ``-ex`` by
       inserting ``set +ex`` at the start of the script. But it's not
       recommended.

.. step:: Cmd

   Runs arbitrary command in the container. The argument provided must be
   a YAML list. For example::

       setup:
       - !Ubuntu trusty
       - !Cmd ["apt-get", "install", "-y", "python"]

    You may use YAMLy features to get complex things. To run complex python
    code you may use::

        setup:
        - !Cmd
          - python
          - -c
          - |
            import socket
            print("Builder host", socket.gethostname())

    Or to get behavior similar to :step:`Sh` command, but with different shell:

        setup:
        - !Cmd
          - /bin/bash
          - -exc
          - |
            echo this is a bash script

.. step:: Download

   Downloads file and puts it somewhere in the file system.

   Example::

       - !Download
         url: https://jdbc.postgresql.org/download/postgresql-9.4-1201.jdbc41.jar
         path: /opt/spark/lib/postgresql-9.4-1201.jdbc41.jar

   .. note:: This step does not require any download tool to be installed in
      the container. So may be used to put static binaries into container
      without a need to install the system.

   Options:

   url
     (required) URL to download file from
   path
     (required) Path where to put file. Should include the file name (vagga
     doesn't try to guess it for now). Path may be in ``/tmp`` to be used only
     during container build process.
   mode
     (default '0o644') Mode (permissions) of the file. May be used to make
     executable bit enabled for downloaded script

   .. warning:: The download is cached similarly to other commands. Currently
      there is no way to control the caching. But it's common practice to
      publish every new version of archive with different URL (i.e. include
      version number in the url itself)


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

.. step:: Copy

   Copy file or directory into the container. Useful either to put build
   artifacts from temporary location into permanent one, or to copy files
   from the project directory into the container.

   Example::

        setup:
        - !Copy
          source: /work/config/nginx.conf
          path: /etc/nginx/nginx.conf

   For directories you might also specify regular expression to ignore::

        setup:
        - !Copy
          source: /work/mypkg
          path: /usr/lib/python3.4/site-packages/mypkg
          ignore-regex: "(~|.py[co])$"

   Symlinks are copied as-is. Path translation is done neither for relative nor
   for absolute symlinks. Hint: relative symlinks pointing inside the copied
   directory work well, as well as absolute symlinks that point to system
   locations.

   .. note:: The command fails if any file name has non-utf-8 decodable names.
      This is intentional. If you really need bad filenames use traditional
      ``cp`` or ``rsync`` commands.

   Options:

   source
     (required) Absolute to directory or file to copy. If path starts with
     ``/work`` files are checksummed to get the version of the container.

   path
     (required) Destination path

   ignore-regex
     (default ``(^|/)\.(git|hg|svn|vagga)($|/)|~$|\.bak$|\.orig$|^#.*#$``)
     Regular expression of paths to ignore. Default regexp ignores common
     revision control folders and editor backup files.

   owner-uid, owner-gid
     (preserved by default) Override uid and gid of files and directories when
     copying. It's expected that most useful case is ``owner-uid: 0`` and
     ``owner-gid: 0`` but we try to preserve the owner by default. Note that
     unmapped users (the ones that don't belong to user's subuid/subgid range),
     will be set to ``nobody`` (65535).

   .. warning:: If the source directory starts with `/work` all the files are
      read and checksummed on each run of the application in the container. So
      copying large directories for this case may influence container startup
      time even if rebuild is not needed.

   This command is useful for making deployment containers (i.e. to put
   application code to the container file system). For this case checksumming
   issue above doesn't apply. It's also useful to enable :opt:`auto-clean` for
   such containers.

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

::

    setup:
    #...
    - !EnsureDir /var/cache/downloads
    - !Sh if [ -d "/var/cache/downloads" ]; then echo "Directory created"; fi;
    - !EnsureDir /creates/parent/directories

.. step:: EmptyDir

   Cleans up a directory. It's similar to the `Remove` but keeps directory
   created.

.. step:: CacheDirs

   Adds build cache directories. Example::

        - !CacheDirs
          /tmp/pip-cache/http: pip-cache-http
          /tmp/npm-cache: npm-cache

   This maps ``/tmp/pip-cache/http`` into the cache directory of the vagga, by
   default it's ``~/.vagga/.cache/pip-cache-http``. This allows to reuse same
   download cache by multiple rebuilds of the container. And if shared cache
   is used also reuses the cache between multiple projects.

   Be picky on the cache names, if file conficts there may lead to unexpected
   build results.

   .. note:: Vagga uses a lot of cache dirs for built-in commands. For example
      the ones described above are used whenever you use ``Py*`` and ``Npm*``
      commands respectively. You don't need to do anything special to use
      cache.


Meta Data
=========

.. step:: Env

   Set environment variables for the build.

   Example::

       setup:
       - !Env HOME: /root

   .. note:: The variables are used only for following build steps, and are
      inherited on the :step:`Container` directive. But they are *not used when
      running* the container.

.. step:: Depends


Sub-Containers
==============

.. step:: Container

.. step:: SubConfig


Node.JS Commands
================

.. step:: NpmInstall

   Example::

        setup:
        - !NpmInstall [babel-loader@6.0, webpack]

   Install a list of node.js packages. If no linux distributions were used yet
   ``!NpmInstall`` installs the latest ``Alpine`` distribution. Node is
   installed automatically and analog of the ``node-dev`` package is also added
   as a build dependency.

.. step:: NpmDependencies

   Works similarly to :step:`NpmInstall` but installs packages from
   ``package.json``. For example::

        - !NpmDependencies

   This installs dependencies and ``devDependencies`` from ``package.json``
   into a container (with ``--global`` flag).

   You may also customize ``package.json`` and install other kinds of
   dependencies::

        - !NpmDependencies
          file: frontend/package.json
          peer: true
          optional: true
          dev: false


   .. note:: Since npm supports a whole lot of different versioning schemes and
      package sources, some features may not work or may not version properly.
      You may send a pull request for some unsupported scheme. But we are going
      to support only the popular ones. Generally, it's safe to assume that we
      support a npmjs.org packages and git repositories with full url.

   .. note:: We don't use ``npm install .`` to execute this command but
      rather use a command-line to specify every package there. It works better
      because ``npm install --global .`` tries to install this specific package
      to the system, which is usually not what you want.


   Options:

   file
       (default ``package.json``) A file to get dependencies from


   package
       (default ``true``) Whether to install package dependencies (i.e. the
       ones specified in ``dependencies`` key)

   dev
       (default ``true``) Whether to install ``devDependencies`` (we assume
       that vagga is mostly used for develoment environments so dev
       dependencies should be on by default)

   peer
       (default ``false``) Whether to install ``peerDependencies``

   bundled
       (default ``true``) Whether to install ``bundledDependencies`` (and
       ``bundleDependencies`` too)

   optional
       (default ``false``) Whether to install ``optionalDependencies``. *By
       default npm tries to install them, but don't fail if it can't install.
       Vagga tries its best to guarantee that environment is the same, so
       dependencies should either install everywhere or not at all.
       Additionally because we don't use "npm install package.json" as
       described earlier we can't reproduce npm's behavior exactly. But
       optional dependencies of dependencies will probably try to install.*

   .. warning:: This is a new command. We can change default flags used, if
      that will be more intuitive for most users.


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

   .. note:: Every time :step:`PipConfig` is specified, options are **replaced**
      rather than *augmented*. In other words, if you start a block of pythonic
      commands with :step:`PipConfig`, all subsequent commands will be executed
      with the same options, no matter which :step:`PipConfig` settings were before.

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
       (default ``[]``) List of URLs to HTML files that need to be parsed
       for links that indicate the packages to be downloaded.

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

   install-python
       (default ``true``) Install python automatically. This will install
       either python2 or python3 with a default version of your selected linux
       distribution. You may set this parameter to ``false`` and install python
       yourself. This flag doesn't disable automatic installation of pip itself
       and version control packages. Note that by default ``python-dev`` style
       packages are as build dependencies installed too.

   python-exe
       (default is either ``python2`` or ``python3`` depending on which command
       is called, e.g. ``Py2Install`` or ``Py3Install``) This allows to change
       executable of python. It may be either just name of the specific python
       interpreter (``python3.5``) or full path. Note, when this is set, the
       command will be called both for ``Py2*`` commands and ``Py3*`` commands.

   .. _pip help: https://pip.readthedocs.org/en/stable/reference/pip_install/


.. step:: Py2Install

.. step:: Py2Requirements

.. step:: Py3Install

.. step:: Py3Requirements

.. not yet implemented

    .. step:: PyFreeze

       Install python dependencies and freeze them.

       .. admonition:: Experimental

          This command is a subject of change at any time, while we are trying to
          figure out how this thing should work.

       Example::

            setup:
            - !Ubuntu trusty
            - !PyFreeze
              freeze-file: "requirements.txt"
              packages: [flask]

       If the file "requirements.txt" exists. It will install the packages listed
       in the file, otherwise it will build temporary container. Run ``pip freeze``
       in the container and store the data in ``requirements.txt``. Then it will
       build the real container.

       The file ``requirements.txt`` is expected to be checked out into version
       control, so everybody gets same dependencies.

       If ``packages`` is changed after ``requirements.txt`` is generated, vagga
       should be able to detect this and regenerate requirements.txt

       Parameters:

       freeze-file
         (default ``requirements.txt``) The file where dependencies will be stored

       requirements
         (optional) The file where original list of dependencies is. This option
         is an alternative to ``packages``

       packages
         (optional) List of python packages to install. Packages may optionally
         contain versions.


       See `the article`__ for motivation for this command

       __ https://medium.com/p/vagga-the-higher-level-package-manager-e49e85fed42a


