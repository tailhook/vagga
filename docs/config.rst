===================
Vagga Configuration
===================

Main vagga configration file is ``vagga.yaml`` it's usually in the root of the
project dir. It can also be in ``.vagga/vagga.yaml`` (but it's not recommended).

The ``vagga.yaml`` has three sections:

* ``containers`` -- description of the containers
* ``commands`` -- a set of commands defined for the project

.. _containers:

Containers
==========

Example of one container defined:

.. code-block:: yaml

  containers:
    sphinx:
      setup:
      - !Ubuntu trusty
      - !Install [python-sphinx, make]

The YAML above defines a container named ``sphinx``, which is built with two
steps: download and unpack ubuntu ``trusty`` base image, and install install
packages name ``python-sphinx, make``  inside the container.

Container parameters:

..
    ``default-command``
        This command is used when running ``vagga _run <container_name>``. Note
        that this command doesn't use ``command-wrapper``, so you may include that
        value explicitly
    ``command-wrapper``
        The wrapper script thats used to run anything inside container. For example
        setting the value to ``/usr/bin/env`` and running ``vagga _run cmd args``
        will actually run ``/usr/bin/env cmd args``. This may be either a string,
        which is treated as single command (e.g. no split by space), or a list.
    ``shell``
        The shell used to run commands with ``run`` key, and for ``vagga _run -S``.
        ``command-wrapper`` is not used for it. This may be either a string,
        which is treated as single command (e.g. no split by space), or a list.
        For usual shell must be ``[/bin/sh, -c]``.

``setup``
    List of steps that is executed to build container. See :ref:`build_commands`
    for more info.

``environ-file``
    The file with environment definitions. Path inside the container. The file
    consists of line per value, where key and value delimited by equals ``=``
    sign. (Its similar to ``/etc/environment`` in ubuntu or ``EnvironmentFile``
    in systemd, but doesn't support commands quoting and line wrapping yet)

``environ``
    The mapping, that constitutes environment variables set in container. This
    overrides ``environ-file`` on value by value basis.

``uids``
    List of ranges of user ids that need to be mapped when container runs.
    User must have some ranges in ``/etc/subuid`` to run this contiainer,
    and total size of all allowed ranges must be larger or equal to the sum of
    sizes of all ranges specified in ``uids`` parameter.  Currenlty vagga
    applies ranges found in ``/etc/subuid`` one by one until all ranges are
    satisfied. It's not always optimal or desirable, we will allow to customize
    mapping in later versions.

    Default value is ``[0-65535]`` which is usually good enough. Unless you
    have smaller number of uids available or run container in container.

``gids``
    List of ranges of group ids that need to be mapped when container runs.
    User must have some ranges in ``/etc/subgid`` to run this contiainer,
    and total size of all allowed ranges must be larger or equal to the sum of
    sizes of all ranges specified in ``gids`` parameter.  Currenlty vagga
    applies ranges found in ``/etc/subgid`` one by one until all ranges are
    satisfied. It's not always optimal or desirable, we will allow to customize
    mapping in later versions.

    Default value is ``[0-65535]`` which is usually good enough. Unless you
    have smaller number of gids available or run container in container.

``volumes``
    The mapping of mount points to the definition of volume. Allows to mount
    some additional filesystems inside the container. See :ref:`volumes` for more
    info. Default is::

        volumes:
            /tmp: !Tmpfs { size: 100Mi, mode: 0o1777 }

    .. note:: You must create a folder for each volume. See :ref:`build_commands` for
       documentation.

``resolv-conf-path``
    The path in container where to copy ``resolv.conf`` from host. If the value
    is ``null``, no file is copied.  Default is ``/etc/resolv.conf``. Its
    useful if you symlink ``/etc/resolv.conf`` to some tmpfs directory in
    ``setup`` and point ``resolv-conf-path`` to the directory.

    .. note:: The default behavior for vagga is to overwrite
       ``/etc/resolv.conf`` inside the container at the start. It's violation
       of read-only nature of container images (and visible for all
       containers). But as we are doing only single-machine development
       environments, it's bearable. We are seeking for a better way without too
       much hassle for the user. But you can use the symlink if it bothers you.


Commands
========

Example of command defined:

.. code-block:: yaml

   commands:
     build-docs: !Command
       description: Build vagga documentation using sphinx
       container: sphinx
       work-dir: docs
       run: make

The YAML above defines a command named ``build-docs``, which is run in
container named ``sphinx``, that is run in ``docs/`` sub dir of project, and
will run command ``make`` in container. So running::

    > vagga build-docs html

Builds html docs using sphinx inside a container.

See commands_ for comprehensive description of how to define commands.

.. _YAML: http://yaml.org
