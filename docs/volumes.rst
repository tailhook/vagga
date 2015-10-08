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
      /dev/shm: !Tmpfs
        size: 100Mi
        mode: 0o1777

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

