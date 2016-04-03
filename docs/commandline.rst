.. highlight:: bash

============
Command Line
============

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

vagga _run CONTAINER CMD ARG...
  run arbitrary command in container defined in vagga.yaml

vagga _build CONTAINER
  Builds container without running a command.

  More useful in the form::

      $ vagga _build --force container_name

  To rebuid container that has previously been built.

vagga _clean
  Removes images and temporary files created by vagga.

  The following command removes containers that are not used by current vagga
  config (considering the state of all files that ``vagga.yaml`` depends on)::

      $ vagga _clean --unused

  There is a faster option for removing unused containers::

      $ vagga _clean --old

  This is different because it only looks at symlinks in ``.vagga/*``. So may
  be wrong (if you changed ``vagga.yaml`` and did not run the command(s)). It's
  faster because it doesn't calculate the hashsums. But the difference in
  speed usually not larger than a few seconds (on large configs). The existence
  of the two commands should probably be treated as a historical accident
  and ``--unused`` variant preferred.

  For other operations and paremeters see ``vagga _clean --help``

vagga _list
  List of commands (similar to running vagga without command)

vagga _version_hash CONTAINER
  Prints version hash for the container. In case the image has not been built
  (or config has been updated since) it should return new hash. But sometimes
  it's not possible to determine the hash in advance. In this case command
  returns an error.

  Might be used in some automation scripts.

vagga _init_storage_dir
  If you have configured a :opt:`storage-dir` in settings, say
  ``/vagga-storage``, when you run ``vagga _init_storage_dir abc`` will create
  a ``/vagga-storage/abc`` and ``.vagga`` with ``.vagga/.lnk`` pointing to
  the directory. The command ensures that the storage dir is not used for any
  other folder.

  This is created for buildbots which tend to clean ``.vagga`` directory on
  every build (like gitlab-ci) or just very often.

vagga _pack_image IMAGE_NAME
  Pack image into the tar archive, optionally compressing and output it into
  stdout (use shell redirection ``> file.tar`` to store it into the file).

  It's very similar to ``tar -cC .vagga/IMAGE_NAME/root`` except it deals with
  file owners and permissions correctly. And similar to running
  ``vagga _run IMAGE_NAME tar -c /`` except it ignores mounted file systems.

.. _vagga_push_image:

vagga _push_image IMAGE_NAME
  Push container image ``IMAGE_NAME`` into the image cache.

  Actually it boils down to packing an image into tar (``vagga _pack_image``)
  and running :opt:`push-image-script`, see the documentation of the setting
  to find out how to configure image cache.


Normal Commands
===============

If :ref:`command<commands>` declared as ``!Command`` you get a command
with the following usage::

    Usage:
        vagga [OPTIONS] some_command [ARGS ...]

    Runs a command in container, optionally builds container if that does not
    exists or outdated. Run `vagga` without arguments to see the list of
    commands.

    positional arguments:
      some_command          Your defined command
      args                  Arguments for the command

    optional arguments:
      -h,--help             show this help message and exit
      -E,--env,--environ NAME=VALUE
                            Set environment variable for running command
      -e,--use-env VAR      Propagate variable VAR into command environment
      --no-build            Do not build container even if it is out of date.
                            Return error code 29 if it's out of date.
      --no-version-check    Do not run versioning code, just pick whatever
                            container version with the name was run last (or
                            actually whatever is symlinked under
                            `.vagga/container_name`). Implies `--no-build`

All the  ``ARGS`` that follow command are passed to the command even if they
start with dash ``-``.


Supervise Commands
==================

If :ref:`command<commands>` declared as ``!Supervise`` you get a command
with the following usage::


    Usage:
        vagga run [OPTIONS]

    Run full server stack

    optional arguments:
      -h,--help             show this help message and exit
      --only PROCESS_NAME [...]
                            Only run specified processes
      --exclude PROCESS_NAME [...]
                            Don't run specified processes
      --no-build            Do not build container even if it is out of date.
                            Return error code 29 if it's out of date.
      --no-version-check    Do not run versioning code, just pick whatever
                            container version with the name was run last (or
                            actually whatever is symlinked under
                            `.vagga/container_name`). Implies `--no-build`

Currently there is no way to provide additional arguments to commands declared
with ``!Supervise``.

The ``--only`` and ``--exclude`` arguments are useful for isolating some
single app to a separate console. For example, if you have ``vagga run``
that runs full application stack including a database, cache, web-server
and your little django application, you might do the following::

    $ vagga run --exclude django

Then in another console::

    $ vagga run --only django

Now you have just a django app that you can observe logs from and restart
independently of other applications.
