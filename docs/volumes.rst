.. highlight:: yaml
.. default-domain:: vagga

.. _volumes:

=======
Volumes
=======

Volumes define some additional filesystems to mount inside container. The
default configuration is similar to the following:

.. code-block:: yaml

    volumes:
      /tmp: !Tmpfs
        size: 100Mi
        mode: 0o1777
      /run: !Tmpfs
        size: 100Mi
        mode: 0o766
        subdirs:
          shm: { mode: 0o1777 }

.. warning:: Volumes are **not** mounted during container build, only when
   some command is run.

Available volume types:

.. volume:: Tmpfs

    Mounts ``tmpfs`` filesystem. There are two parameters for this kind of
    volume:

      * ``size`` -- limit for filesystem size in bytes. You may use
        suffixes ``k, M, G, ki, Mi, Gi`` for bigger units. The ones with ``i``
        are for power of two units, the other ones are for power of ten;
      * ``mode`` -- filesystem mode.
      * ``subdirs`` -- a mapping for subdirectories to create inside tmpfs,
        for example::

         volumes:
            /var: !Tmpfs
                mode: 0o766
                subdirs:
                    lib: # default mode is 0o766
                    lib/tmp: { mode: 0o1777 }
                    lib/postgres: { mode: 0o700 }

        The only property currently supported on a directory is ``mode``.

.. volume:: VaggaBin

    Mounts vagga binary directory inside the container (usually it's contained
    in ``/usr/lib/vagga`` in host system). This may be needed for
    :ref:`network_testing` or may be for vagga in vagga (i.e. container in
    container) use cases.


.. volume:: BindRW

   Binds some folder inside a countainer to another folder. Essentially it's
   bind mount (the ``RW`` part means read-writeable). The path must be
   absolute (inside the container). This directive can't be used to expose
   some directories not already visible. This is often used to put some
   temporary directory in development into well-defined production location.

   For example::

       volumes:
         /var/lib/mysql: !BindRW /work/tmp/mysql

   There are currently two prefixes for :volume:`BindRW`:

   * `/work` -- which uses directory inside the project directory
   * `/volumes` -- which uses one of the volumes defined in settings
     (:opt:`external-volumes`)

   The behavior of vagga when using any other prefix is undefined.

.. volume:: Snapshot

   Create a ``tmpfs`` volume, copy contents of the original folder to the
   volume. And then mount the filesystem in place of the original directory.

   This allows to pre-seed the volume at the container build time, but make
   it writeable and throwable.

   Example::

        volumes:
            /var/lib/mysql: !Snapshot

   .. note:: Every start of the container will get it's own copy. Even every
      process in `!Supervise` mode will get own copy. It's advised to keep
      container having a snapshot volume only for single purpose (i.e. do not
      use same container both for postgresql and python), because otherwise
      excessive memory will be used.

   Parameters:

   size
     (default ``100Mi``) Size of the allocated ``tmpfs`` volume. Including the
     size of the original contents. This is the limit of how much data you can
     write on the volume.

   owner-uid, owner-gid
     (default is to preserve) The user id of the owner of the directory. If not
     specified the ownership will be copied  from the original

   Additional properties, like the source directory will be added to the later
   versions of vagga
