===============
System Settings
===============

Vagga sometimes hints and if permitted can tune few options on a host system.
This is the reference of the options that vagga can fix.

See :opt:`auto-apply-sysctl` for a table of options and limits.


.. _sysctl-max-user-watches:

Sysctl ``fs.inotify.max_user_watches``
======================================

The inotify_ is used to notify user processes that some file or directory is
changed by another process. It's tweaked by :opt:`expect-inotify-limit`.

It's very useful for the following things:

1. Run processes with automatic restart on reload
2. Run build system and automatically rebuild on file change
3. Start unit tests on each file change

Unfortunately on some systems (namely ubuntu xenial, docker on mac) it's very
common to have a limit of ``8192`` inotify watches. Which is too slow on some
systems.

The error is manifested as:

* ``inotify watch limit reached``
* ``ENOSPC`` / ``No space left on device`` (yes, this is not a typo)
* ``Internal watch failed: watch ENOSPC``
* Some programs just crash (see `#291`_)

.. _#291: https://github.com/tailhook/vagga/issues/291

Tuning it is usually harmless unless the value is too large. Each user watch
`takes up to 1080 bytes`__. So values up to 512K are fine on
most current systems.

__ http://askubuntu.com/questions/154255/how-can-i-tell-if-i-am-out-of-inotify-watches

To tune it (temporarily) you need to run::

    sudo sysctl fs.inotify.max_user_watches=524288

To store for the next reboot you may try to add ``-w``::

    sudo sysctl -w fs.inotify.max_user_watches=524288

But it doesn't work for some linux distributions (hello, NixOS)

Alternatively, you may set :opt:`auto-apply-sysctl`. This tells vagga to
automatically run ``sudo -k sysctl ...`` on your behalf (probably asking for a
password).


.. _inotify: https://en.wikipedia.org/wiki/Inotify
