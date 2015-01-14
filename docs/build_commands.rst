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

The ``set -ex`` in above enables error handling and tracing of commands in
shell it's good to specify them in all scripts.

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

To add some environment arguments to build command use ``!Env``:

.. code-block:: yaml

    setup:
    # ...
    - !Env
      VAR1: value1
      VAR2: value2
    - !Sh "echo $VAR1 / $VAR2"

.. note:: The ``!Env`` command doesn't add environment variables for processes
    run after build. Use ``environ`` setting for that.

Sometimes you want to rebuild container when some file changes. For example
if you have used the file in the build. There is a ``!Depends`` command which
does nothing per se, but add a dependency. The path must be relative to your
project directory (the dir where ``vagga.yaml`` is). For example:

.. code-block:: yaml

   setup:
   # ...
   - !Depends requirements.txt
   - !Sh "pip install -r requirements.txt"


.. _marathon: https://github.com/mesosphere/marathon


