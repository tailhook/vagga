.. highlight:: bash

===============
Tips And Tricks
===============


Faster Builds
=============

There are :ref:`settings` which allow to set common directory for cache for
all projects that use vagga. I.e. you might add the following to
``$HOME/.config/vagga/settings.yaml``:

.. code-block:: yaml

    cache-dir: ~/.cache/vagga/cache

Currently you must create directory by hand.


Multiple Build Attempts
=======================

Despite of all caching vagga does it's usually to slow to rebuild big container
for trying to install single package. You might try something like this::

    $ vagga _run --writeable container_name pip install pyzmq

Note the flag ``--writeable`` or shorter ``-W`` doesn't write into container
itself, but creates a (hard-linked) copy, which is destructed on exit. So to
run multiple commands you might use bash::

    host-shell$ vagga _run -W container bash
    root@localhost:/work# apt-get update
    root@localhost:/work# apt-get install -y something

.. note:: We delete package indexes of ubuntu after container is built. It's
   done to keep image smaller. So you always need ``apt-get update`` step.

Another technique is to use :ref:`dependent_containers`.


Debug Logging
=============

You can enable additional debug logging by setting the environment variable
``RUST_LOG=debug``. For example::

    $ RUST_LOG=debug vagga _build container


I'm Getting "permission denied" Errors
======================================

If when starting vagga you're getting::

    ERROR:container::monitor: Can't run container wrapper: Error executing: permission denied

Then you're probably don't have apropriate kernel option enabled. You may try::

    $ sysctl -w kernel.unprivileged_userns_clone=1

If that works, you should add it to your system startup. If it doesn't
unfortunately it may mean you need to recompile the kernel. It's not that
complex nowadays, but still disturbing.

Anyway if you didn't find specific instructions for your system on
:ref:`installation` page please `report an issue`_ with information of your
distribution (at least ``uname`` and ``/etc/os-release``) so I can add
instructions.

.. _report an issue: https://github.com/tailhook/vagga/issues


How to Debug Slow Build?
========================

There is a log with timings for each step, in container's metadata folder.
The easiest way to view it::

    $ cat .vagga/<container_name>/../timings.log
      0.000   0.000   Start 1425502860.147834
      0.000   0.000   Prepare
      0.375   0.374   Step: Alpine("v3.1")
      1.199   0.824   Step: Install(["alpine-base", "py-sphinx", "make"])
      1.358   0.159   Finish

.. note:: Note the ``/../`` part. It works because ``.vagga/<container_name>``
   is a symlink. Real path is something like
   ``.vagga/.roots/<container_name>.<hash>/timings.log``

First column displays time in seconds since container started building. Second
column is a time of this specific step.

You should also run build at least twice to see the impact of package caching.
To rebuild container run::

    $ vagga _build --force <container_name>


How to Find Out Versions of Installed Packages?
===============================================

You can use typical ``dpkg -l`` or similar command. But since we usually
deinstall ``npm`` and ``pip`` after setting up container for space efficiency
we put package list in contianer metadata. In particular there are following
lists:

* ``alpine-packages.txt`` -- list of packages for Alpine linux
* ``debian-packages.txt`` -- list of packages for Ubuntu/Debian linux
* ``pip2-freeze.txt``/``pip3-freeze.txt`` -- list of python packages, in a
  format directly usable for ``requirements.txt``
* ``npm-list.txt`` -- a tree of npm packages

The files contain list of all packages incuding ones installed implicitly
or as a dependency. All packages have version. Unfortunately format of files
differ.

The files are at parent directory of the container's filesystem, so can be
looked like this::

    $ cat .vagga/<container_name>/../pip3-freeze.txt

Or specific version can be looked::

    $ cat .vagga/.roots/<container_name>.<hash>/pip3-freeze.txt

The latter form is useful to compare with old version of the same container.
