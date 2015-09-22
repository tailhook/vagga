.. highlight:: bash

==================
Vagga Command Line
==================

When runnin ``vagga``, it  finds the ``vagga.yaml`` or ``.vagga/vagga.yaml``
file in current working directory or any of its parents and uses that as a
project root directory.

When running ``vagga`` without arguments it displays a short summary of which
commands are defined by ``vagga.yaml``, like this::

    $ vagga
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
* ``vagga _clean`` -- removes images and temporary files created by vagga. To
  fully remove ``.vagga`` directory you can run ``vagga _clean --everything``.
  For other operations see ``vagga _clean --help``
* ``vagga _list`` -- list of commands (including builtin ones when using
  ``--builtin`` flag)
* ``vagga _version_hash`` -- prints version hash for the container, might be
  used in some automation scripts
