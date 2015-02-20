.. _build_commands:

==============
Build Commands
==============


Build commands are tagged values in your container definition. For example:

.. code-block:: yaml

    containers:
      ubuntu:
        setup:
        - !Ubuntu trusty
        - !Install [python]

This contains two build commands ``!Ubuntu`` and ``!Install``. They mostly
run sequentially, but some of them are interesting, for example
``!BuildDeps`` installs package right now, but also removes package at
the end of the build to keep container smaller and cleaner.



Generic Installers
==================

To run arbitrary shell command use ``!Sh``:

.. code-block:: yaml

   setup:
   - !Ubuntu trusty
   - !Sh "apt-get install -y python"

If you have more than one-liner you may use YAMLy *literal* syntax for it:

.. code-block:: yaml

   setup:
   - !Ubuntu trusty
   - !Sh |
      set -ex
      wget somepackage.tar.gz
      tar -xzf somepackage.tar.gz
      cd somepackage
      make && make install

The ``set -ex`` in example above enables error handling and tracing of commands
in shell. It's good idea to add the line to all scripts.

To run ``!Sh`` you need ``/bin/sh``. If you don't have shell in container you
may use ``!Cmd`` that runs command directly:

.. code-block:: yaml

   setup:
   # ...
   - !Cmd [/usr/bin/python, '-c', 'print "hello from build"']

To install a package of any (supported) linux distribution just use
``!Install`` command:

.. code-block:: yaml

   containers:

     ubuntu:
       setup:
       - !Ubuntu trusty
       - !Install [python]

     ubuntu-precise:
       setup:
       - !Ubuntu precise
       - !Install [python]

     alpine:
       setup:
       - !Alpine v3.1
       - !Install [python]

Occasionally you need some additional packages to use for container building,
but not on final machine. Use ``!BuildDeps`` for them:

.. code-block:: yaml

    setup:
    - !Ubuntu trusty
    - !Install [python]
    - !BuildDeps [python-dev, gcc]
    - !Sh "make && make install"

The ``python-dev`` and ``gcc`` packages from above will be removed after
building whole container.

To add some environment arguments to subsequent build commands use ``!Env``:

.. code-block:: yaml

    setup:
    # ...
    - !Env
      VAR1: value1
      VAR2: value2
    - !Sh "echo $VAR1 / $VAR2"

.. note:: The ``!Env`` command doesn't add environment variables for processes
    run after build. Use ``environ`` setting for that.

.. _depends:

Sometimes you want to rebuild container when some file changes. For example
if you have used the file in the build. There is a ``!Depends`` command which
does nothing per se, but add a dependency. The path must be relative to your
project directory (the dir where ``vagga.yaml`` is). For example:

.. code-block:: yaml

   setup:
   # ...
   - !Depends requirements.txt
   - !Sh "pip install -r requirements.txt"

To download and unpack tar archive use ``!Tar`` command:

.. code-block:: yaml

   setup:
   - !Tar
     url: http://something.example.com/some-project-1.0.tar.gz
     sha256: acd1234...
     path: /
     subdir: some-project-1.0

Only ``url`` field is mandatory. The ``path`` is target path to unpack into,
and ``subdir`` is a dir inside tar file. By default ``path`` is root of new
filesystem. The ``subdir`` is a dir inside the tar file, if omitted whole tar
archive will be unpacked.  You *can* use ``!Tar`` command to download and
unpack the root filesystem from scratch.

There is a shortcut to download tar file and build and install from there,
which is ``!TarInstall``:

.. code-block:: yaml

   setup:
   - !TarInstall
     url: http://static.rust-lang.org/dist/rust-0.12.0-x86_64-unknown-linux-gnu.tar.gz
     sha256: abcd1234...
     subdir: rust-0.12.0-x86_64-unknown-linux-gnu
     script: ./install.sh --prefix=/usr

Only the ``url`` is mandatory here too. The ``script`` is by default
``./configure --prefix=/usr; make; make install``. It's run in ``subdir`` of
unpacked archive. If ``subdir`` is omitted it's run in the *only* subdirectory
of the archive. If archive contains more than one directory and ``subdir`` is
empty, it's an error, however you may use ``.`` as ``subdir``.

To remove some data from the image after building use ``!Remove`` command:

.. code-block:: yaml

   setup:
   # ...
   - !Remove /var/cache/something

To clean directory but ensure that directory exists use ``!EmptyDir`` command:

.. code-block:: yaml

   setup:
   # ...
   - !EmptyDir /tmp

.. note:: The ``/tmp`` directory is declared as ``!EmptyDir`` implicitly for
   all containers.

To ensure that directory exists use ``!EnsureDir`` command. It's very often
used for future mount points:

.. code-block:: yaml

   setup:
   # ...
   - !EnsureDir /sys
   - !EnsureDir /dev
   - !EnsureDir /proc

.. note:: The ``/sys``, ``/dev`` and ``/proc`` directories are created
   automatically for all containers.

Sometimes you want to keep some cache between builds of container or similar
containers. Use ``!CacheDirs`` for that:

.. code-block:: yaml

   setup
   # ...
   - !CacheDirs { "/var/cache/apt": "apt-cache" }

Mutliple directories may be specified at once.

.. warning:: The "apt-cache" name is a name of the directory like
   ``.vagga/.cache/apt-cache``. So the directory is shared both between
   all the containers and all the different builders (not only same versions
   of the single container). In case user enabled ``shared-cache`` the folder
   will be also shared between containers of different projects.


.. _marathon: https://github.com/mesosphere/marathon


Ubuntu
======


To install base ubuntu system use:

.. code-block:: yaml

    setup:
    - !Ubuntu trusty

Potentially any ubuntu release instead of ``trusty`` should work.

To install any ubuntu package use generic ``!Install`` command:

.. code-block:: yaml

    setup:
    - !Ubuntu trusty
    - !Install python

Many interesting ubuntu packages are in the "universe" repository, you may add
it by series of ``!UbuntuRepo`` commands (see below), but there is shortcut
``!UbuntuUniverse``:

.. code-block:: yaml

   setup:
   - !Ubuntu trusty
   - !UbuntuUniverse
   - !Install [checkinstall]

The ``!UbuntuRepo`` command adds additional repository. For example to add
marathon_ repository you may write:


.. code-block:: yaml

    setup:
    - !Ubuntu trusty
    - !UbuntuRepo
      url: http://repos.mesosphere.io/ubuntu
      suite: trusty
      components: [main]
    - !Install [mesos, marathon]

This effectively adds repository and installs ``mesos`` and ``marathon``
packages.

.. note:: Probably the key for repository should be added to be able to install
    packages.


Alpine
======


To install base alpine system use:

.. code-block:: yaml

    setup:
    - !Alpine v3.1

Potentially any alpine version instead of ``v3.1`` should work.

To install any alpine package use generic ``!Install`` command:

.. code-block:: yaml

    setup:
    - !Alpine v3.1
    - !Install python


Npm Installer
=============

You can build somewhat default nodejs environment using ``!NpmInstall``
command. For example:

.. code-block:: yaml

    setup:
    - !Ubuntu trusty
    - !NpmInstall [react-tools]

All node packages are installed as ``--global`` which should be expected. If
no distribution is specified before the ``!NpmInstall`` command, the implicit
``!Alpine v3.1`` (in fact the latest version) will be executed.

.. code-block:: yaml

   setup:
   - !NpmInstall [react-tools]

So above should just work as expected if you don't need any special needs. E.g.
it's usually perfectly ok if you only use node to build static scripts.

The following ``npm`` features are supported:

* Specify ``package@version`` to install specific version **(recommended)**
* Use ``git://`` url for the package. In this case git will be installed for
  the duration of the build automatically
* Bare ``package_name`` (should be used only for one-off environments)

Other forms may work, but are unsupported for now.


.. note:: The ``npm`` and additional utilities (like ``build-essential`` and
    ``git``) will be removed after end of container building. You must
    ``!Install`` them explicitly if you rely on them later.


Python Installer
================

There are two separate commands for installing packages for python2 and
python3. Here is a brief example:

.. code-block:: yaml

    setup:
    - !Ubuntu trusty
    - !Py2Install [sphinx]

Currently packages are installed by system pip_. We consider this an
implementation detail and will use latest pip_ in future. The ``python-dev``
headers are installed for the time of the build too. Both ``python-dev`` and
``pip`` are removed when installation is finished.

The following ``pip`` package specification formats are supported:

* The ``package_name==version`` to install specific version **(recommended)**
* Bare ``package_name`` (should be used only for one-off environments)
* The ``git+`` and ``hg+`` links (the git and mercurial are installed as build
  dependency automatically)

All other forms may work but not supported. Specifying command-line arguments
instead of package names is not supported. To configure pip use ``!PipConfig``
directive. In the example there are full list of parameters:

.. code-block:: yaml

    setup:
    - !Ubuntu trusty
    - !PipConfig
      index-urls: ["http://internal.pypi.local"]
      find-links: ["http://internal.additional-packages.local"]
      dependencies: true
    - !Py2Install [sphinx]

They should be self-descriptive. Note unlike in pip command line we use single
list both for primary and "extra" indexes. See pip documentation for more info
about options

.. note:: By default ``dependencies`` is false. Which means pip is run with
   ``--no-deps`` option. Which is recommended way for setting up isolated
   environments anyway. Even ``setuptools`` are not installed by default.
   To see list of dependencies and their versions you may use
   ``pip freeze`` command.

.. _pyreq:

Better way to specify python dependencies is to use "requirements.txt":

.. code-block:: yaml

    setup:
    - !Ubuntu trusty
    - !Py3Requirements "requirements.txt"

This works the same as ``Py3Install`` including auto-installing of version
control packages and changes tracking. I.e. It will rebuild container when
"requirements.txt" change. So ideally in python projects you may use two lines
above and that's it.

The ``Py2Requirements`` command exists too.

.. note:: The "requirements.txt" is checked semantically. I.e. empty lines
   and comments are ignored. In current implementation the order of items
   is significant but we might remove this restriction in the future.


.. _pip: http://pip.pypa.io

.. _dependent_containers:

Dependent Containers
====================

Sometimes you want to build on top of another container. For example container
for running tests might be based on production container, but it might add some
test utils. Use ``!Container`` command for that:

.. code-block:: yaml

   container:
     base:
       setup:
       - !Ubuntu trusty
       - !Py3Install [django]
     test:
       setup:
       - !Container base
       - !Py3Install [nosetests]

It's also sometimes useful to freeze some part of container and test next build
steps on top of of it. For example:

.. code-block:: yaml

   container:
     temporary:
       setup:
       - !Ubuntu trusty
       - !TarInstall
         url: http://download.zeromq.org/zeromq-4.1.0-rc1.tar.gz
     web:
       setup:
       - !Container _temporary
       - !Py3Install [pyzmq]

In this case when you try multiple different versions of pyzmq, the zeromq
itself will not be rebuilt. When you're done, you can append build steps and
remove the ``temporary`` container.





