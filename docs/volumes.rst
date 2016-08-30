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

    Mounts a ``tmpfs`` filesystem.

    Options:

    size
      Limit for filesystem size in bytes. You may use suffixes
      ``k, M, G, ki, Mi, Gi`` for bigger units. The ones with ``i``
      are for power of two units, the other ones are for power of ten

    mode
      The mode (permission bits) of the root directory for a new filesystem

    subdirs
      A mapping for subdirectories to create inside tmpfs, for example::

         volumes:
            /var: !Tmpfs
                mode: 0o766
                subdirs:
                    lib: # default mode is 0o766
                    lib/tmp: { mode: 0o1777 }
                    lib/postgres: { mode: 0o700 }

      The only property currently supported on a directory is ``mode``

    files
      A mapping of a file to it's contents to write into tmpfs filesystem.
      This is similar to :step:`Text` build step. Directories for files
      are not created automatically. Use ``subdirs`` to create one.

      Example::

        volumes:
          /run: !Tmpfs
            size: 100Mi
            files:
              docker.sock: ""
          /run/docker.sock: !BindRO /volumes/docker.sock

    Both ``subdirs`` and ``files`` are often used as mountpoints for some
    :volume:`BindRW` and :volume:`BindRO` directories.

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

.. volume:: BindRO

   Read-only bind mount of a folder inside a container to another folder. See
   :volume:`BindRW` for more info.

.. volume:: Empty

   Mounts an empty read-only directory. Technically mounts a new `Tmpfs` system
   with minimal size and makes it read-only. Useful if you want to hide some
   built-in directory or subdirectory of ``/work`` from the container. For
   example::

        volumes:
          /tmp: !Empty

   Note, that hiding ``/work`` itself is not supported. You may hide a
   subdirectory though::

        volumes:
          /work/src: !Empty


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

   container
     Copy contents of the folder of another container. Path to the source folder
     is the same as the mountpoint.

   owner-uid, owner-gid
     (default is to preserve) The user id of the owner of the directory. If not
     specified the ownership will be copied  from the original

   Additional properties, like the source directory will be added to the later
   versions of vagga

.. volume:: Container

   Mount a root file system of other container as a volume.

   Example::

       containers:
         app:
           setup:
           - !Ubuntu xenial
           ...
         deploy-tools:
           setup:
           - !Alpine v3.4
           - !Install [rsync]
           volumes:
             /mnt: !Container app

   This may be useful to deploy the container without installing anything to
   the host file system. E.g. you can ``rsync`` the container's file system
   to remote host. Or ``tar`` it (but better use :cmd:`_pack_image` or
   :cmd:`_push_image` for that). Or do other fancy things.

   Unless you know what are you doing both containers should share same
   :opt:`uids` and :opt:`gids`.

   .. note:: Nothing is mounted on top of container's file system. I.e.
      ``/dev``, ``/proc`` and ``/sys`` directories are empty. So you probably
      can't chroot into the filesystem in any sensible way. But having that
      folders empty is actually what is useful for use cases like deploying.


.. volume:: Persistent

   Makes a writable directory just for this container. It's similar to
   :volume:`BindRW` but creates a volume inside `.vagga/.volumes`

   Example::

     commands:
       postgres: !Command
         volumes:
           /var/lib/postgres: !Persistent { name: "postgres" }
         run: ...

   Or the shorter form::

     volumes:
       /var/lib/postgres: !Persistent "postgres"

   There are a few reasons to use :volume:`Persistent` over :volume:`BindRW`:

   1. User don't need to create the directories
   2. When running vagga in VM it's a common practice to use more efficient
      (or more featureful, like supporting hardlinks) filesystem for `.vagga`
   3. It may be a little bit clearer than throwing all that writable stuff
      into workdir (for example your `.vagga` is already in `.gitignore`)

   Options:

   name
     **(required)** Name of the volume. Multiple containers using same name
     will mount same volume (same instance of volume). Multiple volumes in
     single container may reference same volume too. We currently don't
     support mounting subvolumes but we may do in future.

   init-command
     The name of the command that is used for initializing volume. Technically
     command is inserted into the prerequisites of every other command that
     uses this volume. The command must have this volume either in container
     or in command definition. When command is first run, it has an empty
     directory at the mount point of the volume. If command fails we
     immediately stop running dependent commands, which effectively means no
     other command can run with the volume mounted.

     It's usually good idea to name the command starting with underscore, so
     it doesn't show in the list of commands to use for daily work.

     If volume is already initialized and command is run in a normal way
     (using `vagga xx` or as a prerequisite), it's run as any other command.
     But if the volume is not initialized it will be run with the temporary
     directory at a volume mount point which will be committed to a volume on
     success. Basically this allows to debug the command easily.

     If the same `init-command` is repeated in multiple volumes it is run only
     once (so you must initialize all the volumes that depend on it).
     Multiple volumes that have different `init-command` values can only be
     used in the command definition (not in container), because otherwise
     it's impossible to establish an initialization order.

     Currently `!Supervise` commands can't be used to initialize a volume, but
     we may lift this limitation in the future.

   To remove volumes that were created but had been removed since than run::

     vagga _clean --unused-volumes

   To remove all volumes (they will be created on the next run)::

     vagga _clean --volumes
