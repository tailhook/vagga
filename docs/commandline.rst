.. highlight:: bash

============
Command Line
============

When running ``vagga``, it  finds the ``vagga.yaml`` file in current working
directory or any of its parents and uses that as a project root directory
(alternative files are supported too, see below).

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


Full list of files that mark directory as vagga's project:

1. ``vagga.yaml`` primary and preferred one
2. ``.vagga/vagga.yaml`` as an alternative to ``vagga.yaml`` (useful if you
   don't want to commit it to a git)
3. ``vagga.local.yaml`` or ``.vagga.local.yaml`` or ``.vagga/local.yaml``
   which contain additional :opt:`mixins` also mark project directory even
   if no ``vagga.yaml`` is present (since vagga 0.8.1)


Multiple Commands
=================

Since vagga 0.6 there is a way to run multiple commands at once::

    $ vagga -m cmd1 cmd2

This is similar to running::

    $ vagga cmd1 && vagga cmd2

But there is one key difference: **containers needed to run all the commands
are built beforehand**. This has two consequences:

1. When containers need to be rebuilt, they are rebuilt first, then you see
   the output of both commands in sequence (no container build log in-between)
2. If container for command 2 depends on side-effects of running command 1
   (i.e. container contains a binary built by command 1), you will get wrong
   results. In that case you should rely on shell to do the work (for example
   in the repository of vagga itself ``vagga -m make test`` is **not** the
   right way, the right is ``vagga make && vagga test``)

Obviously you can't pass any arguments to either of commands when running
``vagga -m``, this is also the biggest reason of why you can't run built-in
commands (those starting with underscore) using the option. But you can use
global options, and they influence all the commands, for example::

    $ vagga --environ DISPLAY:0 -m clean_profile run_firefox


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

  To rebuild a container that has previously been built.

vagga _clean
  Removes images and temporary files created by vagga.

  The following command removes containers that are not used by current vagga
  config (considering the state of all files that ``vagga.yaml`` depends on)::

      $ vagga _clean --unused

  Another for removes containers which were not uses for some time::

      $ vagga _clean --unused --at-least 10days

  This is faster as it only checks timestamps of the containers. Each time
  any command in a container is run, we update timestamp. This is generally
  more useful than bare ``--unused``, because it allows to keep multiple
  versions of same container, which means you can switch between branches
  rapidly.

  There an old and deprecated option for removing unused containers::

      $ vagga _clean --old

  This is different because it only looks at symlinks in ``.vagga/*``. So may
  be wrong (if you changed ``vagga.yaml`` and did not run the command(s)). It's
  faster because it doesn't calculate the hashsums. But the difference in
  speed usually not larger than a few seconds (on large configs). The existence
  of the two commands should probably be treated as a historical accident
  and ``--unused`` variant preferred.

  For other operations and parameters see ``vagga _clean --help``

vagga _list
  List of commands (similar to running vagga without command)

vagga _version_hash CONTAINER
  Prints version hash for the container. In case the image has not been built
  (or config has been updated since) it should return new hash. But sometimes
  it's not possible to determine the hash in advance. In this case command
  returns an error.

  Might be used in some automation scripts.

vagga _init_storage_dir
  **Deprecated**. Use :opt:`storage-subdir-from-env-var` instead.

  If you have configured a :opt:`storage-dir` in settings, say
  ``/vagga-storage``, when you run ``vagga _init_storage_dir abc`` will create
  a ``/vagga-storage/abc`` and ``.vagga`` with ``.vagga/.lnk`` pointing to
  the directory. The command ensures that the storage dir is not used for any
  other folder (unless ``--allow-multiple`` is specified).

  This is created for buildbots which tend to clean ``.vagga`` directory on
  every build (like gitlab-ci) or just very often.

  Since vagga 0.6 there is ``--allow-multiple`` flag, that allows to keep
  shared subdirectory for multiple source directories. This is useful for CI
  systems which use different build directories for different builds.

  .. warning:: While simultanenous builds of different source directories, with
     the same subdirectory should work most of the time, this functionality
     still considered exerimental and may have some edge cases.

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

vagga _base_dir
  Displays (writes to stdout) directory where active ``vagga.yaml`` is.

vagga _relative_work_dir
  Displays (writes to stdout) current working directory relative to the
  base directory. Basically, this means that
  ``$(vagga _base_dir)/$(vagga _relative_work_dir)`` is current working
  directory.

  When current working directory contains ``vagga.yaml`` this command returns
  empty string (output still contains a newline), not a single dot, as one
  may expect.

.. _update_symlinks:

vagga _update_symlinks
  **This functionality is experimental**. Some details can change in future.

  Creates a set of symlinks in your home directory (`~/.vagga/cmd`) and in
  current vagga directory (`.vagga/.cmd`) which point to commands named in
  vagga. Symlinks are created to the current vagga binary (which is resolved
  using ``readlink /proc/self/exe`` not, ``argv[0]``).

  These directories can be added to ``PATH`` either in your shell or in
  your text editor, IDE, or any other kind of shell. Or you can pass them
  to scripts which allow customization
  (``make RSYNC=/myproj/.vagga/.cmd/rsync``).

  Only comands which have ``symlink-name`` are linked with the name specified
  in the parameter. So you make create a hidden (underscored) name for some
  public command.

  There are two directories, so basically two modes of operation:

  1. User home directory ``~/.vagga/cmd``. It meant to use for utilities
     you're going to use in multiple projects. When running such a command in
     some project dir, exact command from this project dir will be invoked. So
     if you run ``flake8`` (a linter for python), correct version of linter
     for this project will be run. If you ``cd`` to another project, correct
     version of the tool with specific plugins and python interpreter will be
     picked there immediately.

  2. Project directory ``proj/.vagga/.cmd``. This directory may be used to
     specify utility directly or to point your IDE to in project settings. It's
     not recommended to add this directory to your search ``PATH``.

  Note: for (1) it's expected that single version of vagga is used for all of
  the projects, which is usually the case.

  .. versionadded:: 0.7.1


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
