.. highlight:: yaml

.. _build_steps:

===========================
Build Steps (The Reference)
===========================

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

   Simple and straightforward way to install Ubuntu release.

   Example::

       setup:
       - !Ubuntu xenial

   The value is single string having the codename of release ``xenial``,
   ``trusty`` and ``precise`` known to work at the time of writing.

   The Ubuntu images are updated on daily basis. But vagga downloads and
   caches the image. To update the image that was downloaded by vagga you need
   to clean the cache.

   .. note:: This is shortcut install that enables all the default that are
      enabled in :step:`UbuntuRelease`. You can switch to ``UbuntuRelease`` if
      you need fine-grained control of things.

.. step:: UbuntuRelease

   This is more exensible but more cumbersome way to setup ubuntu (comparing
   to :step:`Ubuntu`). For example to install trusty you need::

   - !UbuntuRelease { codename: trusty }

   (note this works since vagga 0.6, previous versions required `version` field
   shich is now deprecated).

   You can also setup non-LTS release of different architecture::

   - !UbuntuRelease { codename: vivid, arch: i386 }

   All options:

   codename
     Name of the ubuntu release. Like `xenial` or `trusty`. Either this field
     or `url` field must be specified. If both are specified `url` take
     precedence.

   url
     Url to specific ubuntu image to download. May be any image, including
     `server` and `desktop` versions, but `cloudimg` is recommended. This
     must be filesystem image (i.e usuallly ending with `root.tar.gz`) not
     `.iso` image.

     Example: ``http://cloud-images.ubuntu.com/xenial/current/xenial-server-cloudimg-amd64-root.tar.gz``

   arch
     The architecture to install. Defaults to ``amd64``.

   keep-chfn-command
     (default ``false``) This may be set to ``true`` to enable
     ``/usr/bin/chfn`` command in the container. This often doesn't work on
     different host systems (see `#52
     <https://github.com/tailhook/vagga/issues/52>`_ as an example). The
     command is very rarely useful, so the option here is for completeness
     only.

   eatmydata
     (default ``true``) Install and enable ``libeatmydata``. This does **not**
     literally eat your data, but disables all ``fsync`` and ``fdatasync``
     operations during container build. This works only on distributions
     where we have tested it: ``xenial``, ``trusty``, ``precise``. On other
     distributions the option is ignored (but may be implemented in future).

     The ``fsync`` system calls are used by ubuntu package management tools to
     secure installing each package, so that on subsequent power failure your
     system can boot. When building containers it's both the risk is much
     smaller and build starts from scratch on any kind of failure anyway, so
     partially written files and directories do not matter.

     I.e. don't disable this flag unless you really want slow processing, or
     you have some issues with LD_PRELOAD'ing the library.

     .. note:: On ``trusty`` and ``precise`` this also enables ``universe``
        repository by default.

   version
     The verison of ubuntu to install. This must be digital ``YY.MM`` form,
     not a code name.

     **Deprecated**. Supported versions: ``12.04``,
     ``14.04``, ``14.10``, ``15.10``, ``16.04``. Other version will not work.
     This field will also be removed at some point in future.


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
         suite: xenial
         components: [main]
       - !Install [python3.5]

   See :step:`UbuntuPPA` for easier way for dealing specifically with PPAs.

   Options:

   url
     Url to the repository. Default is the mirror url from the current ubuntu
     distribution.

   suite
     Suite of the repository. The common practice is that the suite is named just
     like the codename of the ubuntu release. For example ``xenial``. Default is
     the codename of the current distribution.

   components
     List of the components to fetch packages from. Common practice to have a
     ``main`` component. So usually this setting contains just single
     element ``components: [main]``. **Required**.

   trusted
     Marks repository as trusted. Usually useful for installing unsigned packages
     from local repository. Default is ``false``.

.. step:: UbuntuPPA

   A shortcut to :step:`UbuntuRepo` that adds named PPA. For example, the
   following::

       - !Ubuntu xenial
       - !AptTrust keys: [5BB92C09DB82666C]
       - !UbuntuPPA fkrull/deadsnakes
       - !Install [python3.5]

   Is equivalent to::

       - !Ubuntu xenial
       - !UbuntuRepo
         url: http://ppa.launchpad.net/fkrull/deadsnakes/ubuntu
         suite: xenial
         components: [main]
       - !Install [python3.5]

.. step:: UbuntuUniverse

   The singleton step. Just enables an "universe" repository::

   - !Ubuntu xenial
   - !UbuntuUniverse
   - !Install [checkinstall]


Alpine Commands
===============

.. step:: Alpine

::

   setup:
   - !Alpine v3.5

.. step:: AlpineRepo

   Adds arbitrary alpine repository. For example to add testing repository::

     - !AlpineRepo
       url: http://nl.alpinelinux.org/alpine/
       branch: edge
       repo: testing
       tag: testing
     - !Install [app@testing]

   Options:

   url
     Url to the repository. Default is the mirror url from the current alpine
     distribution.

   branch
     Branch of the repository. For example ``v3.4``, ``edge``. Default is
     the version of the current alpine distribution.

   repo
     Repository to fetch packages from. For example ``main``, ``community``,
     ``testing``. **Required**.

   tag
     Tag for this repository. Alpine package manager will now
     by default only use the untagged repositories. Adding a tag to
     specific package will prefer the repository with that tag.
     To add a tag just put ``@tag`` after the package name. For example::

       - !AlpineRepo
         branch: edge
         repo: testing
         tag: testing
       - !Install [graphicsmagick@testing]


Distribution Commands
=====================

These commands work for any linux distributions as long as distribution is
detected by vagga. Latter basically means you used :step:`Alpine`,
:step:`Ubuntu`, :step:`UbuntuRelease` in container config (or in parent
config if you use :step:`SubConfig` or :step:`Container`)

.. step:: Repo

   Adds official repository to the supported linux distribution. For example::

     setup:
     - !Ubuntu xenial
     - !Repo xenial/universe
     - !Repo xenial-security/universe
     - !Repo xenial-updates/universe

     setup:
     - !Ubuntu xenial
     - !Repo universe # The same as "xenial/universe"

     setup:
     - !Alpine v3.5
     - !Repo edge/testing

     setup:
     - !Alpine v3.5
     - !Repo community # The same as "v3.5/community"

.. step:: Install

::

    setup:
    - !Ubuntu xenial
    - !Install [gcc, gdb]        # On Ubuntu, equivalent to `apt-get install gcc gdb -y`
    - !Install [build-essential] # `apt-get install build-essential -y`
    # Note that `apt-get install` is run 2 times in this example


.. step:: BuildDeps

::

    setup:
    - !Ubuntu xenial
    - !BuildDeps [wget]
    - !Sh echo "We can use wget here, but no curl"
    - !BuildDeps [curl]
    - !Sh echo "We can use wget and curl here"
    # Container built. Now, everything in BuildDeps(wget and curl) is removed from the container.


Generic Commands
================

.. step:: Sh

    Runs arbitrary shell command, for example::

        - !Ubuntu xenial
        - !Sh "apt-get install -y package"

    If you have more than one-liner you may use YAMLy *literal* syntax for it::

        setup:
        - !Alpine v3.5
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
       - !Ubuntu xenial
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

   Or to get behavior similar to :step:`Sh` command, but with different shell::

       setup:
       - !Cmd
         - /bin/bash
         - -exc
         - |
           echo this is a bash script

.. step:: RunAs

   Runs arbitrary shell command as specified user (and/or group), for example::

      - !Ubuntu xenial
      - !RunAs
         user-id: 1
         script: |
           python -c "import os; print(os.getuid())"

   Options:

   script
      (required) Shell command or script to run

   user-id
      (default ``0``) User ID to run command as. If the ``external-user-id`` is
      omitted this has same effect like using ``sudo -u``.

   external-user-id
      (optional) See :ref:`explanation of external-user-id <external-user-id>`
      for ``!Command`` as it does the same.

   group-id
      (default ``0``) Group ID to run command as.

   supplementary-gids
      (optional) The list of group ids of the supplementary groups.
      By default it's an empty list.

   work-dir
      (default ``/work``) Directory to run script in.

   isolate-network
      (default ``false``) See
      :ref:`explanation of isolate-network <isolate-network>`
      for ``!Supervise`` command type.

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

     .. note:: Since vagga 0.6 we allow to unpack local paths starting
        with ``/volumes/`` as file on one of the volumes configured in settings
        (:opt:`external-volumes`). This is exprimental, and requires every user
        to update their setthings before building a container. Still may be
        useful for building company-internal things.

   path
     (default ``/``). Target path where archive should be unpacked to. By
     default it's a root of the filesystem.

   subdir
     (default ``.``) Subdirectory inside the archive to extract. ``.`` extracts
     the root of the archive.

   sha256
     (optional) Sha256 hashsum of the archive. If real hashsum is different this
     step will fail.

   **This command may be used to populate the container from scratch**

.. step:: TarInstall

   Similar to :step:`Tar` but unpacks archive into a temporary directory and
   runs installation script.

   Example::

       setup:
       - !TarInstall
         url: https://static.rust-lang.org/dist/rust-1.10.0-x86_64-unknown-linux-gnu.tar.gz
         script: ./install.sh --prefix=/usr


   url
     **Required**. The url or a path of the archive to fetch. If the url
     startswith dot ``.`` it's treated as a file name relative to the project
     directory. Otherwise it's a url of the file to download.

   subdir
     (optional) Subdirectory which command is run in. May be ``.`` to run
     command inside the root of the archive.

     The common case is having a single directory in the archive,
     and that directory is used as a working directory for script by default.

   sha256
     (optional) Sha256 hashsum of the archive. If real hashsum is different this
     step will fail.

   script
     The command to use for installation of the archive. Default is effectively
     a ``./configure --prefix=/usr && make && make install``.

     The script is run with ``/bin/sh -exc``, to have better error hadling
     and display. Also this means that dash/bash-compatible shell should be
     installed in the previous steps under path ``/bin/sh``.

.. step:: Unzip

   Unpacks zip archive into container's filesystem.

   All options are the same as for :step:`Tar` step.

   Example::

       - !Unzip
         url: https://services.gradle.org/distributions/gradle-3.1-bin.zip
         path: /opt/gradle
         subdir: gradle-3.1

.. step:: Git

   Check out a git repository into a container. This command doesn't require
   git to be installed in the container.

   Example::

        setup:
        - !Alpine v3.5
        - !Install [python3]
        - !Git
          url: git://github.com/tailhook/injections
          path: /usr/lib/python3.5/site-packages/injections

   (the example above is actually a bad idea, many python packages will work
   just from source dir, but you may get improvements at least by precompiling
   ``*.pyc`` files, see :step:`GitInstall`)


   Options:

   url
      (required) The git URL to use for cloning the repository

   revision
      (optional) Revision to checkout from repository. Note if you don't
      specify a revision, the latest one will be checked out on the first
      build and then cached indefinitely

   branch
      (optional) A branch to check out. Usually only useful if revision is
      not specified

   path
      (required) A path where to store the repository.


.. step:: GitInstall

   Check out a git repository to a temporary directory and run script. This
   command doesn't require git to be installed in the container.

   Example::

        setup:
        - !Alpine v3.5
        - !Install [python, py-setuptools]
        - !GitInstall
          url: git://github.com/tailhook/injections
          script: python setup.py install

   Options:

   url
      (required) The git URL to use for cloning the repository

   revision
      (optional) Revision to checkout from repository. Note if you don't
      specify a revision, the latest one will be checked out on the first
      build and then cached indefinitely

   branch
      (optional) A branch to check out. Usually only useful if revision is
      not specified

   subdir
      (default root of the repository) A subdirectory of the repository to
      run script in

   script
      (required) A script to run inside the repository. It's expected that
      script does compile/install the software into the container. The script
      is run using `/bin/sh -exc`


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

   include-regex
     (default ``None``)
     Regular expression of paths to include. When path matches both ignore and
     include expressions it will be ignored. Also note that if
     ``include-regex`` matches only the folder, no contents will be included.
     For example ``patches/.*\.sql$`` will copy all ``patches`` directories with
     all ``.sql`` files inside them.

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

   Remove file or a directory from the container and **keep it clean on the end
   of container build**. Useful for removing cache directories.

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

   Rebuild the container when a file changes. For example:

   .. code-block:: yaml

      setup:
      # ...
      - !Depends requirements.txt
      - !Sh "pip install -r requirements.txt"

   The example is not the best one, you could use :step:`Py3Requirements` for
   the same task.

   Only the hash of the contents of a file is used in versioning the container
   not an owner or permissions.  Consider adding the :opt:`auto-clean` option
   if it's temporary container that depends on some generated file (sometimes
   useful for tests).


Sub-Containers
==============

.. step:: Container

   Build a container based on another container::

       container:
         base:
           setup:
           - !Ubuntu xenial
           - !Py3Install [django]
         test:
           setup:
           - !Container base
           - !Py3Install [nose]

   There two known use cases of functionality:

   1. Build test/deploy containers on top of base container (example above)
   2. Cache container build partially if you have to rebuild last commands
      of the container frequently

   In theory, the container should behave identically as if the commands would
   be copy-pasted to the `setup` fo dependent container, but sometimes things
   doesn't work. Known things:

   1. The packages in a :step:`BuildDeps` are removed
   2. :step:`Remove` and :step:`EmptyDir` will empty the directory
   3. :step:`Build` with `temporary-mount` is not mounted

   If you have any other bugs with container nesting report in the bugtracker.

   .. note:: :step:`Container` step doesn't influence ``environ`` and
      ``volumes`` as all other options of the container in any way. It only
      somewhat replicate ``setup`` sequence. We require whole environment
      be declared manually (you you can use YAMLy aliases)


.. step:: SubConfig

    This feature allows to generate (parts of) ``vagga.yaml`` for the
    container.  For example, here is how we use a docker2vagga_ script to
    transform ``Dockerfile`` into vagga config:

    .. code-block:: yaml

      docker-parser: ❶
        setup:
        - !Alpine v3.5
        - !Install [python]
        - !Depends Dockerfile ❷
        - !Depends docker2vagga.py ❷
        - !Sh 'python ./docker2vagga.py > /docker.yaml' ❸

      somecontainer:
        setup:
        - !SubConfig
          source: !Container docker-parser ❶
          path: docker.yaml ❹
          container: docker-smart ❺

    Few comments:

    * ❶ -- container used for build, it's rebuilt automatically as a dependency for
      "somecontainer"
    * ❷ -- normal dependency rules apply, so you must add external files that are
      used to generate the container and vagga file in it
    * ❸ -- put generated vagga file inside a container
    * ❹ -- the "path" is relative to the source if the latter is set
    * ❺ -- name of the container used *inside* a "docker.yaml"

    .. _docker2vagga: https://github.com/tailhook/vagga/blob/master/tests/subconfig/docker2vagga.py

    .. warning:: The functionality of ``!SubConfig`` is experimental and is a
       subject to change in future. In particular currently the ``/work`` mount
       point and current directory used to build container are those of initial
       ``vagga.yaml`` file. It may change in future.

    The ``!SubConfig`` command may be used to include some commands from another
    file without building container. Just omit ``generator`` command:

    .. code-block:: yaml

       subdir:
         setup:
         - !SubConfig
           path: subdir/vagga.yaml
           container: containername

    The YAML file used may be a partial container, i.e. it may contain just few
    commands, installing needed packages. The other things (including the name of
    the base distribution) can be set by original container:

    .. code-block:: yaml

        # vagga.yaml
        containers:
          ubuntu:
            setup:
            - !Ubuntu xenial
            - !SubConfig
              path: packages.yaml
              container: packages
          alpine:
            setup:
            - !Alpine v3.5
            - !SubConfig
              path: packages.yaml
              container: packages

        # packages.yaml
        containers:
          packages:
            setup:
            - !Install [redis, bash, make]


.. step:: Build

   This command is used to build some parts of the container in another one.
   For example::

        containers:
          webpack: ❶
            setup:
            - !NpmInstall [webpack]
            - !NpmDependencies
          jsstatic:
            setup:
            - !Container webpack ❶
            - !Copy ❷
                source: /work/frontend
                path: /tmp/js
            - !Sh |
                cd /tmp/js
                webpack --output-path /var/javascripts
            auto-clean: true ❸
          nginx:
            setup:
            - !Alpine v3.5
            - !Install [nginx]
            - !Build
              container: jsstatic
              source: /var/javascripts
              path: /srv/www

   Note the following things:

   * ❶ -- We use separate container for npm *dependencies* so we don't have
     to rebuild it on each change of the sources
   * ❷ -- We copy javascript sources into our temporary container.
     The important part of copying operation is that all the sources are hashed
     and versioned when copying. So container will be rebuild on source
     changes. Since we don't need sources in the container we just put them in
     temporary folder.
   * ❸ -- The temporary container is cleaned automatically (there is low chance
     that it will ever be reused)

   Technically it works similar to ``!Container`` except it doesn't apply
   configuration from the source container and allows to fetch only parts of
   the resulting container.

   Another motivating example is building a package::

        containers:
          pkg:
            setup:
            - !Ubuntu xenial
            - !Install [build-essential]
            - !EnsureDir /packages
            - !Sh |
                checkinstall --pkgname=myapp --pakdir=/packages make
            auto-clean: true
          nginx:
            setup:
            - !Ubuntu xenial
            - !Build
              container: pkg
              source: /packages
              temporary-mount: /tmp/packages
            - !Sh dpkg -i /tmp/packages/mypkg_0.1.deb

   Normal versioning of the containers apply. This leads to the following
   consequences:

   * Putting multiple :step:`Build` steps with the same ``container`` will
     build container only once (this way you may extract multiple folders from
     the single container).
   * Despite the name ``Build`` dependencies are not rebuilt.
   * The :step:`Build` command itself depends only on the container but on on
     the individual files. You need to ensure that the source container is
     versioned well (sometimes you need :step:`Copy` or :step:`Depends` for
     the task)

   Options:

   container
        (required) Name of the container to build and to extract data from

   source
        (default ``/``) Source directory (absolute path inside the source
        container) to copy files from

   path
        Target directory (absolue path inside the resulting container) to copy
        (either ``path`` or ``temporary-mount`` required)

   temporary-mount
        A directory to mount ``source`` into. This is useful if you don't want
        to copy files, but rather want to use files from there. The directory
        is created automatically if not exists, but not parent directories.
        It's probably good idea to use a subdirectory of the temporary dir,
        like ``/tmp/package``. The mount is **read-only** and persists until
        the end of the container build and is not propagated through
        :step:`Container` step.


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

   .. note:: Packages installed this way (as well as those installed by
       ``!NpmDependencies`` are located under ``/usr/lib/node_modules``. In
       order for node.js to find them, one should set the environment variable
       ``NODE_PATH``, making the example become

       Example::

            setup:
            - !NpmInstall [babel-loader@6.0, webpack]
            environ:
              NODE_PATH: /usr/lib/node_modules

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

.. step:: NpmConfig

   The directive configures various settings of npm commands above.
   For example, you may want to turn off automatic nodejs installation so
   you can use custom oversion of it::

       - !NpmConfig
           install_node: false
           npm_exe: /usr/local/bin/npm
       - !NpmInstall [webpack]

   .. note:: Every time :step:`NpmConfig` is specified, options are
      **replaced** rather than *augmented*. In other words, if you start a
      block of npm commands with :step:`NpmConfig`, all subsequent
      commands will be executed with the same options, no matter which
      :step:`NpmConfig` settings were before.

   All options:

   npm-exe
        (default is ``npm``) The npm command to use for
        installation of packages.

   install-node
        (default ``true``) Whether to install nodejs and npm automatically.
        Setting the option to ``false`` is useful for setting up custom
        version of the node.js.


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

   Installs python package for Python 2.7 using pip. Example:

   .. code-block:: yaml

        setup:
        - !Ubuntu xenial
        - !Py2Install [sphinx]

   We always fetch latest pip for installing dependencies. The ``python-dev``
   headers are installed for the time of the build too. Both ``python-dev``
   and ``pip`` are removed when installation is finished.

   The following ``pip`` package specification formats are supported:

   * The ``package_name==version`` to install specific version
     **(recommended)**
   * Bare ``package_name`` (should be used only for one-off environments)
   * The ``git+`` and ``hg+`` links (the git and mercurial are installed as
     build dependency automatically), since vagga 0.4 ``git+https`` and
     ``hg+https`` are supported too (required installing ``ca-certificates``
     manually before)

   All other forms may work but not supported. Specifying command-line
   arguments instead of package names is not supported.

   See :step:`Py2Requirements` for the form that is both more convenient and
   supports non-vagga installations better.

   .. note:: If you configure ``python-exe`` in :step:`PipConfig` there is no
      difference between :step:`Py2Install` and :step:`Py3Install`.

.. _pip: http://pip.pypa.io

.. step:: Py2Requirements

   This command is similar to :step:`Py2Install` but gets package names from
   the file. Example:

   .. code-block:: yaml

        setup:
        - !Ubuntu xenial
        - !Py2Requirements "requirements.txt"

   See :step:`Py2Install` for more details on package installation and
   :step:`PipConfig` for more configuration.

.. step:: Py3Install

   Same as :step:`Py2Install` but installs for Python 3.x by default.

   .. code-block:: yaml

        setup:
        - !Alpine v3.5
        - !Py3Install [sphinx]

   See :step:`Py2Install` for more details on package installation and
   :step:`PipConfig` for more configuration.

.. step:: Py3Requirements

   This command is similar to :step:`Py3Install` but gets package names from
   the file. Example:

   .. code-block:: yaml

        setup:
        - !Alpine v3.5
        - !Py3Requirements "requirements.txt"

   See :step:`Py2Install` for more details on package installation and
   :step:`PipConfig` for more configuration.

.. not yet implemented

    .. step:: PyFreeze

       Install python dependencies and freeze them.

       .. admonition:: Experimental

          This command is a subject of change at any time, while we are trying to
          figure out how this thing should work.

       Example::

            setup:
            - !Ubuntu xenial
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


PHP/Composer Commands
=====================

.. note:: PHP/Composer support was recently added to vagga, some things may
   change as we gain experience with the tool.

.. step:: ComposerInstall

   Example::

        setup:
        - !Alpine v3.5
        - !ComposerInstall ["phpunit/phpunit:~5.2.0"]

   Install a list of php packages using ``composer global require --prefer-dist
   --update-no-dev``. Packages are installed in ``/usr/local/lib/composer/vendor``.

   Binaries are automatically installed to ``/usr/local/bin`` by Composer so
   they are available in your PATH.

   Composer itself is located at ``/usr/local/bin/composer`` and available in
   your PATH as well. After container is built, the Composer executable is no
   longer available.

.. step:: ComposerDependencies

   Install packages from ``composer.json`` using ``composer install``. For
   example::

        - !ComposerDependencies

   Similarly to :step:`ComposerInstall`, packages are installed at
   ``/usr/local/lib/composer/vendor``, including those listed at ``require-dev``,
   as Composer default behavior.

   Options correspond to the ones available to the ``composer install`` command
   line so refer to `composer cli docs`_ for detailed info.

   Options:

   working_dir
       (default ``None``) Use the given directory as working directory

   dev
       (default ``true``) Whether to install ``require-dev`` (this is Composer
       default behavior).

   prefer
       (default ``None``) Preferred way to download packages. Can be either
       ``source`` or ``dist``. If no specified, will use Composer default
       behavior (use ``dist`` for stable).

   ignore_platform_reqs
       (default ``false``) Ignore ``php``, ``hhvm``, ``lib-*`` and ``ext-*``
       requirements.

   no_autoloader
       (default ``false``) Skips autoloader generation.

   no_scripts
       (default ``false``) Skips execution of scripts defined in
       ``composer.json``.

   no_plugins
       (default ``false``) Disables plugins.

   optimize_autoloader
       (default ``false``) Convert PSR-0/4 autoloading to classmap to get a
       faster autoloader.

   classmap_authoritative
       (default ``false``) Autoload classes from the classmap only. Implicitly
       enables ``optimize_autoloader``.

   .. _composer cli docs: https://getcomposer.org/doc/03-cli.md#install

.. step:: ComposerConfig

   The directive configures various settings of composer commands above.
   For example, you may want to use hhvm instead of php::

      - !ComposerConfig
        install_runtime: false
        runtime_exe: /usr/bin/hhvm
      - !ComposerInstall [phpunit/phpunit]

   .. note:: Every time :step:`ComposerConfig` is specified, options are
      **replaced** rather than *augmented*. In other words, if you start a
      block of composer commands with :step:`ComposerConfig`, all subsequent
      commands will be executed with the same options, no matter which
      :step:`ComposerConfig` settings were before.

   All options:

   runtime_exe
        (default ``/usr/bin/php``) The command to use for running Composer. When
        setting this option, be sure to specify the full path for the binary. A
        symlink to the provided value will be created at ``/usr/bin/php`` if it
        not exists, otherwise, ``/usr/bin/php`` will remain the same.

   install_runtime
        (default ``true``) Whether to install the default runtime (php)
        automatically. Setting the option to ``false`` is useful when using
        hhvm, for example.

   install_dev
        (default ``false``) Whether to install development packages (php-dev).
        Defaults to false since it is rare for php projects to build modules and
        it may require manual configuration.

   include_path
        (default ``.:/usr/local/lib/composer``) Set ``include_path``. This option
        overrides the default ``include_path`` instead of appending to it.

   keep_composer
        (default ``false``) If set to ``true``, the composer binary will not be
        removed after build.

   vendor_dir
        (default ``/usr/local/lib/composer/vendor``) The directory where composer
        dependencies will be installed.

   .. note:: Setting ``install_runtime`` to false still installs Composer.


Ruby Commands
=============

.. note:: Ruby support is recently added to the vagga some things may change as
   we gain experience with the tool.

.. step:: GemInstall

   Example::

        setup:
        - !Ubuntu xenial
        - !GemInstall [rake]

   Install a list of ruby gems using ``gem install --bindir /usr/local/bin
   --no-document``.

   The ``--bindir`` option instructs ``gem`` to install binaries in ``/usr/local/bin``
   so they are available in your PATH.

.. step:: GemBundle

   Install gems from ``Gemfile`` using ``bundle install --system --binstubs
   /usr/local/bin``. For example::

        - !GemBundle

   Options correspond to the ones available to the ``bundle install`` command
   line, so refer to `bundler documentation`_ for detailed info.

   Options:

   gemfile
       (default ``Gemfile``) Use the specified gemfile instead of Gemfile.

   without
       (default ``[]``) Exclude gems that are part of the specified named group.

   trust_policy
       (default ``None``) Sets level of security when dealing with signed gems.
       Accepts `LowSecurity`, `MediumSecurity` and `HighSecurity` as values.

   .. _bundler documentation: http://bundler.io/bundle_install.html

.. step:: GemConfig

   The directive configures various settings of ruby commands above::

      - !GemConfig
           install_ruby: true
           gem_exe: gem
           update_gem: true
       - !GemInstall [rake]

   .. note:: Every time :step:`GemConfig` is specified, options are
      **replaced** rather than *augmented*. In other words, if you start a
      block of ruby commands with :step:`GemConfig`, all subsequent
      commands will be executed with the same options, no matter which
      :step:`GemConfig` settings were before.

   All options:

   install_ruby
        (default ``true``) Whether to install ruby.

   gem_exe
        (default ``/usr/bin/gem``) The rubygems executable.

   update_gem
        (default ``true``) Whether to update rubygems itself.

   .. note:: If you set ``install_ruby`` to false you will also have to provide
      rubygems if needed.

   .. note:: If you set ``gem_exe``, vagga will no try to update rubygems.
