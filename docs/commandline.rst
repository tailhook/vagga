==================
Vagga Command Line
==================

When runnin ``vagga``, it  finds the ``vagga.yaml`` or ``.vagga/vagga.yaml``
file in current working directory or any of its parents and uses that as a
project root directory.

When running ``vagga`` without arguments it displays a short summary of which
commands are defined by ``vagga.yaml``, like this::

    > vagga
    Available commands:
        run                 Run mysample project
        build-docs          Build documentation using sphinx

Refer to :ref:`commands` for more information of how to define commands for
vagga.

There are also builtin commands. All builtin commands start with underscore
``_`` character to be clearly distinguished from user-defined commands.

Builtin Commands
================

All commands have ``--help``, so we don't duplicate all command-line flags
here

* ``vagga _run CONTAINER CMD ARG...`` -- run arbitrary command in container
  defined in vagga.yaml
* ``vagga _build CONTAINER`` -- builds container without running a command
* ``vagga _chroot DIR CMD ARG...`` -- runs a command in arbitrary folder
  This is analogous to a ``chroot`` command, except it uses namespaces to
  allow it to run without root privileges, it also creates and mounts system
  directories (``/proc``, ``/sys``, ...) like vagga usually do. This is mostly
  used inside builder scripts not daily tasks.
* ``vagga _setvariant NAME VALUE`` -- store in ``.vagga/settings.yaml`` the
  variant to be used, instead of default (yet can be overriden by ``-v``).
  See :ref:`variants` for more info.
