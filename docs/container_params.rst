.. default-domain:: vagga

Container Parameters
====================

.. opt:: setup

    List of steps that is executed to build container. See
    :ref:`build_commands` and :ref:`build_steps` for more info.

.. opt:: environ-file

    The file with environment definitions. Path inside the container. The file
    consists of line per value, where key and value delimited by equals ``=``
    sign. (Its similar to ``/etc/environment`` in ubuntu or ``EnvironmentFile``
    in systemd, but doesn't support commands quoting and line wrapping yet)

.. opt:: environ

    The mapping, that constitutes environment variables set in container. This
    overrides ``environ-file`` on value by value basis.

.. opt:: uids

    List of ranges of user ids that need to be mapped when the container runs.
    User must have some ranges in ``/etc/subuid`` to run this container,
    and the total size of all allowed ranges must be larger or equal to the sum of
    sizes of all the ranges specified in ``uids`` parameter.  Currently vagga
    applies ranges found in ``/etc/subuid`` one by one until all ranges are
    satisfied. It's not always optimal or desirable, we will allow to customize
    mapping in later versions.

    Default value is ``[0-65535]`` which is usually good enough. Unless you
    have a smaller number of uids available or run container in container.

.. opt:: gids

    List of ranges of group ids that need to be mapped when the container runs.
    User must have some ranges in ``/etc/subgid`` to run this container,
    and the total size of all allowed ranges must be larger or equal to the sum of
    sizes of all the ranges specified in ``gids`` parameter.  Currently vagga
    applies ranges found in ``/etc/subgid`` one by one until all ranges are
    satisfied. It's not always optimal or desirable, we will allow to customize
    mapping in later versions.

    Default value is ``[0-65535]`` which is usually good enough. Unless you
    have a smaller number of gids available or run container in container.

.. opt:: volumes

    The mapping of mount points to the definition of volume. Allows to mount
    some additional filesystems inside the container. See :ref:`volumes` for more
    info. Default is:

    .. code-block:: yaml

        volumes:
            /tmp: !Tmpfs { size: 100Mi, mode: 0o1777 }

    .. note:: You must create a folder for each volume. See :ref:`build_commands` for
       documentation.

.. opt:: resolv-conf-path

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

.. opt:: hosts-file-path

    The path in container where to copy ``/ets/hosts`` from host. If the value
    is ``null``, no file is copied. Default is ``/etc/hosts``. The setting
    intention is very similar to :opt:`resolv-conf-path`, so the same
    considerations must be applied.

.. opt:: auto-clean

    (experimental) Do not leave multiple versions of the container lying around.
    Removes the old container version after the new one is successfully build. This is
    mostly useful for containers which depend on binaries locally built (i.e.
    the ones that are never reproduced in future because of timestamp). For
    most containers it's a bad idea because it doesn't allow to switch between
    branches using source-control quickly. Better use ``vagga _clean --old``
    if possible.

.. opt:: image-cache-url

   If there is no locally cached image and it is going to be built, first check
   for the cached image in the specified URL.

   Example::

        image-cache-url: http://example.org/${container_name}.${short_hash}.tar.xz

   To find out how to upload an image see :opt:`push-image-cmd`.

   .. warning:: The url must contain at least `${short_hash}` substitution,
      or otherwise it will ruin the vagga's container versioning.

   .. note:: Similarly to :step:`Tar` command we allow paths starting with
      `.` and `/volumes/` here. It's of limited usage. And we still consider
      this experimental. This may be useful for keeping image cache on network
      file system, presumably on non-public projects.

