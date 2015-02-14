===============
Tips And Tricks
===============


Faster Builds
=============

There are :ref:`settings` which allow to set common directory for cache for
all projects that use vagga. I.e. you might add the following to
``$HOME/.config/vagga/settings.yaml``:

    cache-dir: ~/.cache/vagga/cache

Currently you must create directory by hand.


Multiple Build Attempts
=======================

Despite of all caching vagga does it's usually to slow to rebuild big container
for trying to install single package. You might try something like this:

    vagga _run --writeable container_name pip install pyzmq

Note the flag ``--writeable`` or shorter ``-W`` doesn't write into container
itself, but creates a (hard-linked) copy, which is destructed on exit. So to
run multiple commands you might use bash:

    host-shell$ vagga _run -W container bash
    root@localhost:/work# apt-get update
    root@localhost:/work# apt-get install -y something

.. note:: We delete package indexes of ubuntu after container is built. It's
   done to keep image smaller. So you always need ``apt-get update`` step.


I'm Getting "permission denied" Errors
======================================

If when starting vagga you're getting::

    ERROR:container::monitor: Can't run container wrapper: Error executing: permission denied

Then you're probably don't have apropriate kernel option enabled. You may try::

    sysctl -w kernel.unprivileged_userns_clone=1

If that works, you should add it to your system startup. If it doesn't
unfortunately it may mean you need to recompile the kernel. It's not that
complex nowadays, but still disturbing.

Anyway if you didn't find specific instructions for your system on
:ref:`installation` page please `report an issue`_ with information of your
distribution (at least ``uname`` and ``/etc/os-release``) so I can add
instructions.

.. _report an issue: https://github.com/tailhook/vagga/issues




